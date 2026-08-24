#!/usr/bin/env bash
# madaha build/test helper for this nix environment
export PATH="/nix/store/ig3dxi9sbg0jnkid4s673mnz4kkbfwa4-rustc-bootstrap-1.95.0/bin:$PATH"
export CARGO_HOME="$PWD/.cargo"
export PKG_CONFIG_PATH="/nix/store/99x1jg5622a0k9ch4pwnwaiqvaanlam7-alsa-lib-1.2.16-dev/lib/pkgconfig"
export LIBCLANG_PATH="/nix/store/rpjwnl8zlkqpsi7hkfbm8ap9rpviyz0l-rocm-llvm-clang-unwrapped-6.0.2/lib"
export C_INCLUDE_PATH="/nix/store/20rbjvhw43h4p0z8db4iimbvv1h5bh2s-glibc-2.42-67-dev/include"
export LD_LIBRARY_PATH="/nix/store/6v3x79khipmf0zbn2yivvx7n15jccv47-alsa-lib-1.2.16/lib"
exec cargo "$@"