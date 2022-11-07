{ pkgs, lib, config, ... }:
let
  cfg = config.services.armqr;
  defaultUser = "armqr";
in with lib; {
  options.services.armqr = {
    enable = mkEnableOption "armqr";
    user = {
      type = types.str;
      description = "User to run under";
      default = defaultUser;
    };
    group = {
      type = types.str;
      description = "Group to run under";
      default = defaultUser;
    };
    port = {
      type = types.port;
      description = "Port to listen on";
    };
    stateDir = mkOption {
      type = types.path;
      description = "Path to directory containing the state.";
      default = "/var/lib/armqr";
    };
    passwordFile = mkOption {
      type = types.path;
      description = "Path to file containing the password.";
      default = "/var/lib/secrets/armqr/password";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.armqr = {
      description = "QR Tattoo Redirector";
      wantedBy = [ "network-online.target" ];
      path = with pkgs; [ armqr ];
      environment = {
        ROCKET_STATE_FILE_PATH = cfg.stateFile;
        PASSWORD_FILE_PATH = cfg.passwordFile;
      };

      serviceConfig = {
        User = cfg.user;
        Group = cfg.group;

        ExecStart = ''
          ROCKET_ADMIN_PASSWORD="$(cat "$PASSWORD_FILE_PATH")" armqr
        '';

        NoNewPrivileges = true;
        ReadWritePaths = [ cfg.stateDir ];
        ReadOnlyPaths = [ cfg.passwordFile ];
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        PrivateUsers = false;
        ProtectHostname = true;
        ProtectClock = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        PrivateMounts = true;
      };
    };

    users.users = optionalAttrs (cfg.user == defaultUser) {
      ${defaultUser} = {
        group = cfg.group;
        isSystemUser = true;
      };
    };

    users.groups =
      optionalAttrs (cfg.group == defaultUser) { ${defaultUser} = { }; };
  };
}
