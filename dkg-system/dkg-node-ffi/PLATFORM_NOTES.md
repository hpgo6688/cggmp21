# Platform-Specific Notes

## Cross-Compilation Limitations

### Apple Silicon (M1/M2) Macs

Due to limitations in the `gmp-mpfr-sys` dependency, cross-compilation from Apple Silicon Macs to iOS targets is not supported by default.

**Current Behavior:**
- ✅ macOS builds work normally
- ❌ iOS builds fail with cross-compilation errors

**Workarounds:**

1. **Use force-cross feature** (experimental):
   ```bash
   cargo build --release --target aarch64-apple-ios --features gmp-mpfr-sys/force-cross
   ```

2. **Use Intel Mac for iOS builds**:
   - Build on an Intel Mac for full iOS support
   - Transfer the built libraries to your Apple Silicon Mac

3. **Use CI/CD for iOS builds**:
   - Set up GitHub Actions or similar CI/CD
   - Build iOS libraries in the cloud
   - Download the built libraries

### Intel Macs

Intel Macs support full cross-compilation to iOS targets:
- ✅ macOS builds
- ✅ iOS Simulator (x86_64) builds
- ✅ iOS Device (arm64) builds

## Build Output

### Apple Silicon Mac
```
target/release/libdkg_node_ffi.dylib  # macOS only
```

### Intel Mac
```
target/release/libdkg_node_ffi.dylib                    # macOS
target/x86_64-apple-ios/release/libdkg_node_ffi.a      # iOS Simulator
target/aarch64-apple-ios/release/libdkg_node_ffi.a     # iOS Device
```

## Development Recommendations

1. **For macOS-only development**: Use any Mac
2. **For iOS development on Apple Silicon**: 
   - Use the force-cross feature (experimental)
   - Or use CI/CD for iOS builds
3. **For full iOS support**: Use Intel Mac or CI/CD

## Future Improvements

- Monitor `gmp-mpfr-sys` for better cross-compilation support
- Consider alternative cryptographic libraries with better cross-compilation support
- Implement automated CI/CD for iOS builds 