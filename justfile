#!/usr/bin/env just --justfile

bold := '\033[1m'
normal := '\033[0m'
red := '\033[0;31m'

default:
    @just --list

# run noise binaries
noise mode="server" type="tcp":
    #!/usr/bin/env sh
    case "{{type}}" in
        tcp)
          bin=noise_simple
          features=noise
          ;;
        zenoh)
          case "{{mode}}" in
              server)
                bin=znoise_server
                features=noise,zenoh
                ;;
              client)
                bin=znoise_client
                features=noise,zenoh
                ;;
              *)
                echo -e "{{red}}{{bold}}invalid zenoh noise mode{{normal}}"
                echo -e "    modes: {{bold}}client{{normal}}, {{bold}}server{{normal}}"
                exit 1
                ;;
          esac
          ;;
        tcp-ring)
          bin=noise_simple
          features=noise-ring
          ;;
        *)
          echo -e "{{red}}{{bold}}invalid noise type{{normal}}"
          echo -e "    types: {{bold}}tcp{{normal}}, {{bold}}tcp-ring{{normal}}"
          exit 1
          ;;
    esac
    set -x
    cargo run --color=always --package keybased --profile release --bin $bin --features $features -- --{{mode}}
