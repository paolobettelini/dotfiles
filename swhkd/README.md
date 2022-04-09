## swhkd

This is my configuration for the [swhkdrc](https://github.com/waycrate/swhkd) key-binder.

1. Copy `swhkdrc` to `/etc/swhkd/swhkdrc`
2. `mkdir ~/.config/swhkd`
3. Copy `swhkd.sh` to `~/.config/swhkd/swhkd.sh`
4. `chmod +x ~/.config/swhkd/swhkd.sh`
5. Copy `swhkd.service` to `/etc/systemd/user/swhkd.service`
6. `systemctl --user enable swhkd.service`