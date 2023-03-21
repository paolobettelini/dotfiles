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
sudo pacman -S pipewire wireplumber slurp firefox discord dolphin gwenview alacritty exa
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

## Fish (shell)
```bash
sudo pacman -S fish starship
cd dotfiles

mkdir -p ~/.config/starship
mkdir -p ~/.config/fish

cat fish/config.fish >> ~/.config/fish/config.fish
cp starship/starship.toml ~/.config/starship/
```
The shell must automatically be started by the terminal.

## Application launcher
```bash
sudo pacman -S rofi
```
To start it run `rofi -show drun`.

## Display manager: TODO

## Wallpapers and animated
Il comando `swww`(compilarlo da https://github.com/Horus645/swww) permette di settare i background.
Nel config di hyprland viene eseguito all'inizio `swww img /path/to/wallpaper`.

## Screenshots
```bash
sudo pacman -S grim
```
Grim è un comando che eseguo lo screenshot. Si può mettere il binding
o si può usare un altro softare che si basa su grim per eseguire gli screenshot tipo slameshot.

## Screen recording
```bash
pacman -S wf-recorder
```

## Icone, font e customization di GTK: TODO

## Pacchetti optional
```bash
paru -S gotop # comando di monitoring
pacman -S swayimg # apre le immagini direttamente sul terminal (overlay)
```
Rtfetch: https://github.com/paolobettelini/rtfetch
(Serve cargo e rust per compilare)
Successivamente togliere il commendo da rtfetch nel file di conf di fish