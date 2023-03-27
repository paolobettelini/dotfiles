# A guide to my personal setup

This guide contains everything I do to completly setup my distro.

## Install Arch Linux

Download the Arch Linux ISO from the official website.
Burn the ISO file to a USB medium.
```bash
sudo dd bs=4M if=/path/to/iso of=/dev/usb conv=fdatasync
```
Live CD into the USB and run `archinstall`. Proceed with the installation.
No desktop enviroment is needed.

## Wi-FI
If you need to connect to a Wi-Fi network you can use `iwctl`.
```bash
systemctl enable iwd
systemctl start iwd
iwctl
<connect>
```

## Git
```bash
sudo pacman -S git
git config --global user.name <user>
git config --global user.email <email>
```
TODO: keys

## Installing Paru (AUR helper)
```bash
sudo pacman -S --needed base-devel
git clone https://aur.archlinux.org/paru.git
cd paru
makepkg -si
cd ..
rm -r paru
```

## Download the dotfiles
Download the dotfiles from this repository.
```bash
git clone https://github.com/paolobettelini/dotfiles
```

## Common packages
```bash
sudo pacman -S
    pipewire
    wireplumber
    slurp
    firefox # browser
    discord # discord
    dolphin # file explorer
    gwenview # image viewer
    alacritty # terminal
    exa # alternative to "ls"
    swayimg # Image viewer with overlay
paru -S gotop # system monitoring
```

## Hyprland
```bash
paru -S hyprland-git
```
Copy the dotfiles
```bash
cd dotfiles
mkdir -p ~/.config/hypr
cp hyprland/hyprland.conf to ~/.config/hypr/
cd ..
```

## Rust
```bash
sudo pacman -S cargo
```

## Rtfetch
```bash
git clone https://github.com/paolobettelini/rtfetch
cd rtfetch
cargo build --release
sudo mv target/release/rtfetch /usr/local/bin
```

## Fish (shell)
```bash
sudo pacman -S fish starship
cd dotfiles

mkdir -p ~/.config/starship
mkdir -p ~/.config/fish

cat fish/config.fish >> ~/.config/fish/config.fish
cp starship/starship.toml ~/.config/starship/
```
You can now set fish as the default shell
```bash
chsh -s /bin/fish
```

## Application launcher
```bash
sudo pacman -S rofi
mkdir -p ~/.config/rofi
cd dotfiles
cp rofi/config.rasi ~/.config/rofi/
cd ..
```
To start it run `rofi -show drun`.

## Wallpapers (animated or static)
Create a folder for your wallpapers
```bash
sudo mkdir -p /usr/share/backgrounds
```
Install `swww`
```bash
git clone https://github.com/Horus645/swww
cd swww
cargo build --release
sudo mv target/release/swww /usr/local/bin/
sudo mv target/release/swww-daemon /usr/local/bin/
```
To set the background run
`swww img /path/to/wallpaper`.

## Screenshots
```bash
sudo pacman -S grim
```
`grim` executes a screenshot.
There are also other programs based on grim.

## Screen recording
```bash
pacman -S wf-recorder
```

## SDDM (Display manager)
```bash
sudo pacman -S sddm
sudo systemctl enable sddm

sudo mkdir -p /etc/sddm.conf.d
cd dotfiles
sudo cp sddm/sddm.conf /etc/sddm.conf.d/
```
Download the theme from [here](https://www.opendesktop.org/p/1312658)
```bash
sudo mkdir -p /usr/share/sddm/themes/*
sudo rm -r /usr/share/sddm/themes/*
sudo tar -xzvf ~/Downloads/sugar-candy.tar.gz -C /usr/share/sddm/themes
sudo cp sddm/theme.conf.user /usr/share/sddm/themes/sugar-candy
cd ..
```

## Icons
Download the icons from [here](https://www.gnome-look.org/p/1253385/)
```bash
sudo tar -xf ~/Downloads/Sweet.tar.xz -C /usr/share/icons
```
TODO

## Fonts
Download your font from
[here](https://www.nerdfonts.com/font-downloads)
(DejaVuSansMono Nerd Font)
```bash
sudo mkdir -p /usr/local/share/fonts
sudo unzip -a ~/Downloads/DejaVuSansMono.zip -d /usr/local/share/fonts/
/usr/local/share/fonts/
sudo rm /usr/local/share/fonts/*.txt
sudo rm /usr/local/share/fonts/*.md
```

## Themes

### QT5
```bash
sudo pacman -S qt5ct adwaita-qt5
sudo pacman -S qt6ct adwaita-qt6
qt5ct # select theme
qt6ct # select theme
```

<!--
fish -> exa
hyprland -> swww
fish -> rtfetch
-->