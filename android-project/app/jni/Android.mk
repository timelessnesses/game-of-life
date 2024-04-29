LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE := main

RELATIVE_LIBS_DIR := libs/$(TARGET_ARCH_ABI)
LIB_NAME := libmain.so
GLOBAL_LIBS_DIR := $(LOCAL_PATH)/$(RELATIVE_LIBS_DIR)
LOCAL_SRC_FILES := $(RELATIVE_LIBS_DIR)/$(LIB_NAME)
GLOBAL_LIB_FILE := $(GLOBAL_LIBS_DIR)/$(LIB_NAME)
GLOBAL_VARS_FILE := $(GLOBAL_LIBS_DIR)/vars.sh
CARGO_GLOBAL_PATH := $(LOCAL_PATH)

LOCAL_SHARED_LIBRARIES := SDL2 SDL2_ttf

$(shell mkdir -p $(GLOBAL_LIBS_DIR))
$(shell touch $(GLOBAL_LIB_FILE))

$(shell echo "TARGET_ARCH_ABI=\"$(TARGET_ARCH_ABI)\"" > $(GLOBAL_VARS_FILE))
$(shell echo "TARGET_LIB=\"$(GLOBAL_LIB_FILE)\"" >> $(GLOBAL_VARS_FILE))
$(shell echo "TARGET_ARCH=\"$(TARGET_ARCH)\"" >> $(GLOBAL_VARS_FILE))

include $(PREBUILT_SHARED_LIBRARY)

$(shell echo "TRIPLE=\"$(header_triple_$(TARGET_ARCH))\"" >> $(GLOBAL_VARS_FILE))
$(shell echo "ANDROID_LDFLAGS=\"$(TARGET_LDFLAGS)\"" >> $(GLOBAL_VARS_FILE))
$(shell echo "ANDROID_CC=\"$(TOOLCHAIN_PREFIX)gcc\"" >> $(GLOBAL_VARS_FILE))
$(shell echo "ANDROID_LD_SYSROOT=\"$(SYSROOT_LINK)\"" >> $(GLOBAL_VARS_FILE))

$(CARGO_GLOBAL_PATH)/libs/*/libmain.so : libs/*/libSDL2.so libs/*/libSDL2_image.so
    $(CARGO_GLOBAL_PATH)/build-lib.sh "$@"

$(shell rm $(GLOBAL_LIB_FILE))