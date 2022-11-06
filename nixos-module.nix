{ pkgs, lib, config, ... }:
let cfg = config.services.armqr;
in with lib; {
  options.services.armqr.enable = mkEnableOption "armqr";

  config = mkIf cfg.enable {
    systemd.services.armqr = {
      description = "QR Tattoo Redirector";
      wantedBy = [ "network-online.target" ];
      script = ''
        ${pkgs}/bin/armqr
      '';
    };
  };
}
