## swhkd

This is my configuration for the [swhkdrc](https://github.com/waycrate/swhkd) key-binder.

1. Copy `swhkdrc` to `/etc/swhkd/swhkdrc`
3. Copy `swhkd.sh` to `/usr/local/lib/swhkd.sh`
4. `chmod +x `/usr/local/lib/swhkd.sh`
5. Copy `swhkd.service` to `/etc/systemd/user/swhkd.service`
6. `systemctl --user enable swhkd.service`