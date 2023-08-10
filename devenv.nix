{ pkgs, ... }:

{
  env.GREET = "devenv";

  packages = with pkgs; [ cmake git openssl paho-mqtt-c ];

  languages.rust.enable = true;
}
