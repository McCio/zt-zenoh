#!/usr/bin/env just --justfile

bold := '\033[1m'
normal := '\033[0m'
red := '\033[0;31m'

default:
    @just --list

# run noise binaries
noise mode="server" type="tcp" pkfile="" pubfile="":
    #!/usr/bin/env sh
    args='--{{mode}}'
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
                bin=znoise_client_file
                features=noise,zenoh
                ;;
              *)
                echo -e "{{red}}{{bold}}invalid zenoh noise mode{{normal}}"
                echo -e "    modes: {{bold}}client{{normal}}, {{bold}}server{{normal}}"
                exit 1
                ;;
          esac
          case "{{pkfile}}" in
              "")
                ;;
              *)
                args="${args} "'--key-file {{pkfile}}'
                ;;
          esac
          case "{{pubfile}}" in
              "")
                ;;
              *)
                args="${args} "'--remote-public-key-file {{pubfile}}'
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
    cargo run --color=always --package samp --profile release --bin $bin --features $features -- $args
