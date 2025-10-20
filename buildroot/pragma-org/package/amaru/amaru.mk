################################################################################
#
# Amaru prebuilt binary (local)
#
################################################################################

AMARU_VERSION = local
AMARU_SITE = $(BR2_EXTERNAL_PRAGMA_PATH)/package/amaru/files
AMARU_SITE_METHOD = local
AMARU_LICENSE = MIT
AMARU_LICENSE_FILES = LICENSE

# Déterminer quel binaire installer selon la cible
ifeq ($(BR2_aarch64),y)
AMARU_BIN = amaru-aarch64
else ifeq ($(BR2_arm),y)
AMARU_BIN = amaru-armv7
else ifeq ($(BR2_x86_64),y)
AMARU_BIN = amaru-x86_64
else
$(error "Unsupported architecture for Amaru")
endif

define AMARU_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/$(AMARU_BIN) $(TARGET_DIR)/usr/bin/amaru
endef

$(eval $(generic-package))
