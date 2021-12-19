import 'package:flutter/material.dart';
import 'package:esse/l10n/localizations.dart';

import 'package:esse/apps/jarvis/page.dart';
import 'package:esse/apps/file/list.dart';
import 'package:esse/apps/group/list.dart';
import 'package:esse/apps/domain/page.dart';
import 'package:esse/apps/chat/list.dart';
import 'package:esse/apps/wallet/page.dart';
import 'package:esse/apps/organization/detail.dart';
import 'package:esse/apps/cloud/page.dart';

const List<InnerService> INNER_SERVICES = [
  InnerService.Wallet,
  InnerService.Chat,
  InnerService.GroupChat,
  InnerService.Organization,
  InnerService.Jarvis,
  InnerService.Domain,
  InnerService.Cloud,
];

enum InnerService {
  Chat,
  GroupChat,
  Jarvis,
  Domain,
  Wallet,
  Organization,
  Cloud,
}

extension InnerServiceExtension on InnerService {
  List<String> params(AppLocalizations lang) {
    switch (this) {
      case InnerService.Chat:
        return [lang.contact, lang.contactIntro, 'assets/logo/logo_chat.png'];
      case InnerService.Jarvis:
        return [lang.jarvis, lang.jarvisBio, 'assets/logo/logo_jarvis.png'];
      case InnerService.GroupChat:
        return [lang.groupChat, lang.groupChatIntro, 'assets/logo/logo_group.png'];
      case InnerService.Domain:
        return [lang.domain, lang.domainIntro, 'assets/logo/logo_domain.png'];
      case InnerService.Wallet:
        return [lang.wallet, lang.walletIntro, 'assets/logo/logo_wallet.png'];
      case InnerService.Organization:
        return [lang.organization, lang.organizationIntro, 'assets/logo/logo_organization.png'];
      case InnerService.Cloud:
        return [lang.cloud, lang.cloudIntro, 'assets/logo/logo_cloud.png'];
    }
  }

  Widget callback() {
    switch (this) {
      case InnerService.Chat:
        return ChatList();
      case InnerService.Jarvis:
        return JarvisDetail();
      case InnerService.GroupChat:
        return GroupChatList();
      case InnerService.Domain:
        return DomainDetail();
      case InnerService.Wallet:
        return WalletDetail();
      case InnerService.Organization:
        return OrganizationDetail();
      case InnerService.Cloud:
        return CloudPage();
    }
  }
}
