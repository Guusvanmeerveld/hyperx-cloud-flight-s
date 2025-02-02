# HyperX Cloud Flight S CLI

## Introduction

This is a simple Rust CLI application that interfaces with the HyperX Cloud Flight S headset using the HidAPI. I forked this repository since the original repositories did not satisfy my needs (running with Waybar) or did not support my specific model of headset. Feel free to further fork this repository or create pull requests if you see fit.

## Installation

### Using Nix

#### With flakes

Add the following into the desired flake.nix file.

```nix
{
    inputs.hyperx-cloud-flight-s.url = "github:guusvanmeerveld/hyperx-cloud-flight-s";
}
```

#### Add the overlay

```nix
 {
    config = {
      nixpkgs = {
        overlays = [
            (final: _prev: {
                hyperx-cloud-flight-s = inputs.hyperx-cloud-flight-s.packages."${final.system}".default;
            })
        ];
      };
    };
  }
```

#### Add the package

```nix
{
    environment.systemPackages = [pkgs.hyperx-cloud-flight-s];
    # Needed to allow users to interact with the headset.
    services.udev.packages = [pkgs.hyperx-cloud-flight-s];
}
```

### Supported operating systems

- Nix

## License

This project is licensed under the MIT License - see the [LICENSE.md](/LICENSE.md) file for details

## Other Projects

- [hyperx-cloud-flight-s](https://github.com/Mitnitsky/hyperx-cloud-flight-s) Module for interfacing with HyperX Cloud Flight S.
- [hyperx-cloud-flight](https://github.com/Mitnitsky/hyperx-cloud-flight) Module for interfacing with HyperX Cloud Flight series of headsets.
