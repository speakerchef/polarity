#include <CoreFoundation/CoreFoundation.h>
#include <dlfcn.h>

typedef int (*TCCAccessPreflightFn)(CFStringRef service,
                                    CFDictionaryRef options);

int polarity_audio_capture_permission(void) {
  void *tcc =
      dlopen("/System/Library/PrivateFrameworks/TCC.framework/Versions/A/TCC",
             RTLD_NOW);

  if (tcc == NULL) {
    return -1;
  }

  TCCAccessPreflightFn preflight =
      (TCCAccessPreflightFn)dlsym(tcc, "TCCAccessPreflight");

  if (preflight == NULL) {
    return -1;
  }

  return preflight(CFSTR("kTCCServiceAudioCapture"), NULL);
}
