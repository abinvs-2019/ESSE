use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::GroupId,
        message::{RecvType, SendType},
        primitive::{new_io_error, HandleResult, PeerAddr, Result},
    },
};

use group_chat_types::{ConnectProof, Event, LayerConnect, LayerEvent, LayerResult, PackedEvent};
use tdn_did::Proof;
use tdn_storage::local::DStorage;

use crate::layer::{Layer, Online};
use crate::rpc::{session_connect, session_create, session_last, session_lost, session_suspend};
use crate::session::{connect_session, SessionType};
use crate::storage::{group_chat_db, session_db, write_avatar_sync};

use super::models::{from_network_message, GroupChat, Member, Request};
use super::{add_layer, rpc};

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(..) => {} // Never to here.
        RecvType::Leave(..) => {}   // Never to here. handled in chat.
        RecvType::Result(addr, is_ok, data) => {
            if is_ok {
                let mut layer_lock = layer.write().await;
                handle_connect(mgid, addr, data, &mut layer_lock, &mut results)?;
            } else {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                add_layer(&mut results, mgid, msg);
            }
        }
        RecvType::ResultConnect(addr, data) => {
            let mut layer_lock = layer.write().await;
            if handle_connect(mgid, addr, data, &mut layer_lock, &mut results)? {
                let msg = SendType::Result(0, addr, true, false, vec![]);
                add_layer(&mut results, mgid, msg);
            }
        }
        RecvType::Event(addr, bytes) => {
            let event: LayerEvent =
                postcard::from_bytes(&bytes).map_err(|_| new_io_error("serialize event error."))?;
            handle_event(mgid, addr, event, layer, &mut results).await?;
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            //
        }
    }

    Ok(results)
}

fn handle_connect(
    mgid: GroupId,
    addr: PeerAddr,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<bool> {
    // 0. deserialize result.
    let LayerResult(gcd, height) =
        postcard::from_bytes(&data).map_err(|_e| new_io_error("Deseralize result failure"))?;

    // 1. check group.
    if let Some(group) = load_group(layer.base(), &mgid, &gcd)? {
        // 1.0 check address.
        if group.g_addr != addr {
            return Ok(false);
        }

        // 1.1 get session.
        let session_some =
            connect_session(layer.base(), &mgid, &SessionType::Group, &group.id, &addr)?;
        if session_some.is_none() {
            return Ok(false);
        }
        let sid = session_some.unwrap().id;

        // 1.2 online this group.
        layer
            .running_mut(&mgid)?
            .check_add_online(gcd, Online::Direct(addr), sid, group.id)?;

        // 1.3 online to UI.
        results.rpcs.push(session_connect(mgid, &sid, &addr));

        println!("will sync remote: {}, my: {}", height, group.height);
        // 1.4 sync group height.
        if group.height < height {
            add_layer(results, mgid, sync(gcd, addr, group.height));
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn handle_event(
    mgid: GroupId,
    addr: PeerAddr,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    println!("Got event.......");
    match event {
        LayerEvent::Offline(gcd) => {
            let mut layer_lock = layer.write().await;
            let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
            layer_lock.running_mut(&mgid)?.check_offline(&gcd, &addr);
            drop(layer_lock);
            results.rpcs.push(session_lost(mgid, &sid));
        }
        LayerEvent::Suspend(gcd) => {
            let mut layer_lock = layer.write().await;
            let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
            if layer_lock.running_mut(&mgid)?.suspend(&gcd, false)? {
                results.rpcs.push(session_suspend(mgid, &sid));
            }
            drop(layer_lock);
        }
        LayerEvent::Actived(gcd) => {
            let mut layer_lock = layer.write().await;
            let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
            let _ = layer_lock.running_mut(&mgid)?.active(&gcd, false);
            drop(layer_lock);
            results.rpcs.push(session_connect(mgid, &sid, &addr));
        }
        LayerEvent::CheckResult(ct, supported) => {
            println!("check: {:?}, supported: {:?}", ct, supported);
            results.rpcs.push(rpc::create_check(mgid, ct, supported))
        }
        LayerEvent::CreateResult(gcd, ok) => {
            println!("Create result: {}", ok);
            if ok {
                // get gc by gcd.
                let db = group_chat_db(layer.read().await.base(), &mgid)?;
                if let Some(mut gc) = GroupChat::get(&db, &gcd)? {
                    gc.ok(&db)?;
                    results.rpcs.push(rpc::create_result(mgid, gc.id, ok));

                    // ADD NEW SESSION.
                    let s_db = session_db(layer.read().await.base(), &mgid)?;
                    let mut session = gc.to_session();
                    session.insert(&s_db)?;
                    results.rpcs.push(session_create(mgid, &session));
                }
            }
        }
        LayerEvent::Agree(gcd, info) => {
            println!("Agree..........");
            let base = layer.read().await.base.clone();
            let db = group_chat_db(&base, &mgid)?;
            let (rid, key) = Request::over(&db, &gcd, true)?;

            // 1. add group chat.
            let mut group = GroupChat::from_info(key, info, 0, addr, &base, &mgid)?;
            group.insert(&db)?;

            // 2. ADD NEW SESSION.
            let s_db = session_db(&base, &mgid)?;
            let mut session = group.to_session();
            session.insert(&s_db)?;
            results.rpcs.push(session_create(mgid, &session));

            // 3. update UI.
            results.rpcs.push(rpc::group_agree(mgid, rid, group));

            // 4. try connect.
            let proof = layer
                .read()
                .await
                .group
                .read()
                .await
                .prove_addr(&mgid, &addr)?;
            add_layer(results, mgid, group_chat_conn(proof, addr, gcd));
        }
        LayerEvent::Reject(gcd) => {
            println!("Reject..........");
            let db = group_chat_db(layer.read().await.base(), &mgid)?;
            let (rid, _key) = Request::over(&db, &gcd, true)?;
            results.rpcs.push(rpc::group_reject(mgid, rid));
        }
        LayerEvent::MemberOnline(gcd, mid, maddr) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            results.rpcs.push(rpc::member_online(mgid, gid, mid, maddr));
        }
        LayerEvent::MemberOffline(gcd, mid, ma) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            results.rpcs.push(rpc::member_offline(mgid, gid, mid, ma));
        }
        LayerEvent::Sync(gcd, height, event) => {
            let (sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;

            println!("Sync: height: {}", height);
            let base = layer.read().await.base().clone();
            let db = group_chat_db(&base, &mgid)?;

            match event {
                Event::GroupInfo => {}
                Event::GroupTransfer => {}
                Event::GroupManagerAdd => {}
                Event::GroupManagerDel => {}
                Event::GroupClose => {}
                Event::MemberInfo(mid, maddr, mname, mavatar) => {
                    let id = Member::get_id(&db, &gid, &mid)?;
                    Member::update(&db, &id, &maddr, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_info(mgid, id, maddr, mname));
                }
                Event::MemberJoin(mid, maddr, mname, mavatar, mtime) => {
                    let mut member = Member::new(gid, mid, maddr, mname, false, mtime);
                    member.insert(&db)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_join(mgid, member));
                }
                Event::MemberLeave(_mid) => {}
                Event::MessageCreate(mid, nmsg, mtime) => {
                    println!("Sync: create message start");
                    let base = layer.read().await.base.clone();
                    let msg = from_network_message(height, gid, mid, &mgid, nmsg, mtime, &base)?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));
                    println!("Sync: create message ok");
                    results
                        .rpcs
                        .push(session_last(mgid, &sid, &msg.datetime, &msg.content, true));
                }
            }

            // save event.
            GroupChat::add_height(&db, gid, height)?;
        }
        LayerEvent::Packed(gcd, height, from, to, events) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;

            println!("Start handle sync packed... {}, {}, {}", height, from, to);
            let base = layer.read().await.base().clone();
            handle_sync(
                mgid, gid, gcd, addr, height, from, to, events, base, results,
            )?;
        }
        LayerEvent::Check => {}             // nerver here.
        LayerEvent::Create(..) => {}        // nerver here.
        LayerEvent::Request(..) => {}       // nerver here.
        LayerEvent::RequestResult(..) => {} // nerver here.
        LayerEvent::SyncReq(..) => {}       // Never here.
    }

    Ok(())
}

#[inline]
fn load_group(base: &PathBuf, mgid: &GroupId, gcd: &GroupId) -> Result<Option<GroupChat>> {
    let db = group_chat_db(base, mgid)?;
    GroupChat::get(&db, gcd)
}

pub(crate) fn group_chat_conn(proof: Proof, addr: PeerAddr, gid: GroupId) -> SendType {
    let data =
        postcard::to_allocvec(&LayerConnect(gid, ConnectProof::Common(proof))).unwrap_or(vec![]);
    SendType::Connect(0, addr, None, None, data)
}

fn sync(gcd: GroupId, addr: PeerAddr, height: i64) -> SendType {
    println!("Send sync request...");
    let data = postcard::to_allocvec(&LayerEvent::SyncReq(gcd, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn handle_sync(
    mgid: GroupId,
    fid: i64,
    gcd: GroupId,
    addr: PeerAddr,
    height: i64,
    mut from: i64,
    to: i64,
    events: Vec<PackedEvent>,
    base: PathBuf,
    results: &mut HandleResult,
) -> Result<()> {
    let db = group_chat_db(&base, &mgid)?;

    for event in events {
        let _ = handle_sync_event(&mgid, &fid, from, event, &base, &db, results);
        from += 1;
    }

    if to < height {
        add_layer(results, mgid, sync(gcd, addr, to + 1));
    }

    // update group chat height.
    GroupChat::add_height(&db, fid, to)?;

    Ok(())
}

fn handle_sync_event(
    mgid: &GroupId,
    fid: &i64,
    height: i64,
    event: PackedEvent,
    base: &PathBuf,
    db: &DStorage,
    results: &mut HandleResult,
) -> Result<()> {
    match event {
        PackedEvent::GroupInfo => {
            // TODO
        }
        PackedEvent::GroupTransfer => {
            // TODO
        }
        PackedEvent::GroupManagerAdd => {
            // TODO
        }
        PackedEvent::GroupManagerDel => {
            // TODO
        }
        PackedEvent::GroupClose => {
            // TOOD
        }
        PackedEvent::MemberInfo(_mid, _maddr, _mname, _mavatar) => {
            // TODO
        }
        PackedEvent::MemberJoin(mid, maddr, mname, mavatar, mtime) => {
            if mavatar.len() > 0 {
                write_avatar_sync(&base, &mgid, &mid, mavatar)?;
            }
            let mut member = Member::new(*fid, mid, maddr, mname, false, mtime);
            member.insert(&db)?;
            results.rpcs.push(rpc::member_join(*mgid, member));
        }
        PackedEvent::MemberLeave(_mid) => {
            // TODO
        }
        PackedEvent::MessageCreate(mid, nmsg, time) => {
            let msg = from_network_message(height, *fid, mid, mgid, nmsg, time, base)?;
            results.rpcs.push(rpc::message_create(*mgid, &msg));
        }
        PackedEvent::None => {}
    }

    Ok(())
}
