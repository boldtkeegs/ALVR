## (! Mandatory, apply fix if not applied yet !) Black screen even when SteamVR shows movement, Dashboard not detecting launched ALVR/SteamVR

The Steam runtimes SteamVR runs in break the ALVR driver loaded by SteamVR. This causes the screen to stay black on the headset or an error to be reported that the PipeWire device is missing, or can even result in SteamVR crashing.

### Fix

Add `~/.local/share/Steam/steamapps/common/SteamVR/bin/vrmonitor.sh %command%` to the command-line options of SteamVR (SteamVR -> Manage/Right Click -> Properties -> General -> Launch Options).

This path might differ based on your Steam installation; in that case, SteamVR will not start at all. If this is the case, you can figure out the actual path by going to Steam Settings -> Storage. Then pick the storage location with the star emoji (⭐) and take the path directly above the usage statistics. Prepend this path to `steamapps/common/SteamVR/bin/vrmonitor.sh`. Finally, put this entire path into the SteamVR command-line options instead of the other one.

### Hyprland/Sway/wlroots Qt fix

If you're on Hyprland, Sway, or other wlroots-based Wayland compositors, you might have to additionally prepend `QT_QPA_PLATFORM=xcb` to the command-line.

Related issue:
[[BUG] No SteamVR UI on wlroots-based wayland compositors (sway, hyprland, ...) with workaround](https://github.com/ValveSoftware/SteamVR-for-Linux/issues/637).

## The ALVR driver doesn't get detected by SteamVR (even after vrmonitor fix)

This could be related to Arch AUR package (either installed not for Nvidia on Nvidia-based system (`alvr-nvidia`), or just in general).

### Fix

Try using a launcher or portable .tar.gz release from the Releases page.

## Artifacting, no SteamVR Overlay or graphical glitches in streaming view

This could be related to the AMDVLK or AMDGPU-PRO drivers being present on your system.

If you have AMDVLK installed on your system, it overrides other Vulkan drivers and causes SteamVR to break. Use the `vulkan-radeon` driver (aka RADV) instead.

### Fix

Check if AMDVLK or AMDGPU-PRO are installed by seeing if `ls /usr/share/vulkan/icd.d/ | grep -e amd_icd -e amd_pro` shows anything.
If so, uninstall the AMDVLK and/or AMDGPU-PRO driver(s) from your system (This method may not catch all installations due to distro variations).

On Arch, first install `vulkan-radeon` then uninstall other drivers.

## Failed to create VAAPI encoder

Blocky or crashing streams of gameplay and then an error window on your desktop saying:
> Failed to create VAAPI encoder: Cannot open video encoder codec: Function not implemented. Please make sure you have installed VAAPI runtime.

### Fix

For Fedora:
 * Switch from `mesa-va-drivers` to `mesa-va-drivers-freeworld`. [Guide on how to do so](https://fostips.com/hardware-acceleration-video-fedora/) or [the RPM docs](https://rpmfusion.org/Howto/Multimedia).

For Arch (don't use VAAPI for Nvidia):
 * Follow through [this](https://wiki.archlinux.org/title/Hardware_video_acceleration#Installation) page,
then reboot your machine.

For other distros (e.g. Manjaro):
 * Install the nonfree version of the Mesa/VAAPI drivers that include the proprietary codecs needed for H264/HEVC encoding.

## Nvidia driver version requirements

ALVR requires driver version >=535 and CUDA version >=12.1. If this is not the case, SteamVR or the encoder might not work.

### Fix

Install the required versions of the driver and CUDA.

If an error saying CUDA was not detected persists, try using the latest ALVR nightly release.

## Using ALVR with only integrated graphics

Beware that using **only** integrated graphics for running ALVR is highly inadvisable, as in most cases it will lead to very poor performance (even on more powerful devices like the Steam Deck, it's still very slow). Don't expect things to work perfectly in this case, too, as some older integrated graphics simply might not have the best Vulkan support and might fail to work at all. 

## Hybrid graphics advice

### General advice

If you have a PC and can disable your integrated GPU from BIOS/UEFI, it's highly advised to do so to avoid multiple problems of handling hybrid graphics. If you're on a laptop and it doesn't allow disabling integrated graphics (in most cases), you'll have to resort to the methods below.

### AMD/Intel integrated GPU + AMD/Intel discrete GPU

Prepend `DRI_PRIME=1` to the command-line options of SteamVR and all VR games you intend to play with ALVR.

### AMD/Intel integrated GPU + Nvidia discrete GPU


Prepend `__NV_PRIME_RENDER_OFFLOAD=1 __VK_LAYER_NV_optimus=NVIDIA_only __GLX_VENDOR_LIBRARY_NAME=nvidia` to the command-line options of SteamVR and all VR games you intend to play with ALVR.

If this results in errors such as `error in encoder thread: Failed to initialize vulkan frame context: Invalid argument`, then try adding `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/nvidia_icd.json` to the above.


- Go to `/usr/share/vulkan/icd.d` and make sure `nvidia_icd.json` exists. It may also be under the name `nvidia_icd.x86_64.json`, in which case you should adjust `VK_DRIVER_FILES` accordingly.
- On some distributions packaging older drivers, `VK_ICD_FILENAMES` may need to be used instead.

### SteamVR Dashboard not rendering in VR on Nvidia discrete GPU
If you encounter issues with the SteamVR dashboard not rendering in VR, you may need to run the entire Steam client itself via PRIME render offload. First, close the Steam client completely if you have it open already. You can do so by clicking the Steam dropdown in the top left and choosing Exit. Then from a terminal run: `__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia steam-runtime`.

## Wayland

When using old Gnome (<47) under Wayland, you might need to prepend `WAYLAND_DISPLAY=''` to the SteamVR command-line options to force Xwayland on SteamVR. This fixes an issue with DRM leasing not being available.

## View shakes with SlimeVR

Might be fixed in future updates of ALVR.

### Fix

Start the SlimeVR Server only after you have connected and gotten an image to ALVR at least once.

## 109 Error

The 109 error or others appear.

### Fix

Start Steam first before starting SteamVR through ALVR. If SteamVR is already started, restart it.

## No audio and/or microphone

Even though audio and/or microphone are enabled in presets, audio still can't be heard and/or no one can hear you.

### Fix

Make sure you set `ALVR Audio` and `ALVR Microphone` as default in your devices list **after** connecting the headset. As soon as the headset disconnects, the devices will be removed. If you set them as default, they will be automatically chosen whenever they show up, and you don't need to do it manually ever again.
If you don't appear to have the audio devices, or have PipeWire errors in logs, check if you have `pipewire` >=0.3.49 installed by using the command `pipewire --version`
For older Debian (<=11) or Ubuntu (<=22.04)-based distributions, you can check the [pipewire-upstream](https://github.com/pipewire-debian/pipewire-debian) page to obtain newer PipeWire versions.

## Low AMDGPU performance and stutters

This might be caused by [[PERF] Subpar GPU performance due to wrong power profile mode · Issue #469 · ValveSoftware/SteamVR-for-Linux · GitHub](https://github.com/ValveSoftware/SteamVR-for-Linux/issues/469).

### Fix

Using CoreCtrl is highly advised (install it using your distribution's package management). In its settings, set your GPU to VR profile, as well as CPU to performance profile (if it's an old Ryzen CPU).

## OVR Advanced Settings

Disable the OVR Advanced Settings driver and don't use it with ALVR. It's incompatible and will produce ladder-like latency graphs with very bad shifting vision.

## Bindings not working/high CPU usage due to bindings UI

SteamVR can't properly update bindings, open menus, and/or eats too much CPU.

This issue is caused by SteamVR's webserver spamming requests that stall the Chromium UI and cause it to use a lot of CPU.

### Fix

Apply the following patch: `https://github.com/alvr-org/ALVR-Distrobox-Linux-Guide/blob/main/patch_bindings_spam.sh`
Assuming default path for Arch, Fedora this one-liner is applicable: `curl -s https://raw.githubusercontent.com/alvr-org/ALVR-Distrobox-Linux-Guide/main/patch_bindings_spam.sh | sh -s ~/.steam/steam/steamapps/common/SteamVR`
