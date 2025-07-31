#!/usr/bin/env python3
"""
Correct calculation of Ethereum address from secp256k1 public key
"""

import hashlib
from eth_keys import keys
from eth_utils import to_checksum_address

def calculate_ethereum_address_correct(public_key_hex):
    """
    Calculate Ethereum address from a secp256k1 public key using the correct method
    
    Args:
        public_key_hex (str): Public key in hex format
    
    Returns:
        str: Ethereum address in checksum format
    """
    # Remove 0x prefix if present
    if public_key_hex.startswith('0x'):
        public_key_hex = public_key_hex[2:]
    
    # Handle compressed public key (starts with 02 or 03)
    if public_key_hex.startswith(('02', '03')):
        public_key_bytes = bytes.fromhex(public_key_hex)
        public_key = keys.PublicKey.from_compressed_bytes(public_key_bytes)
    else:
        public_key = keys.PublicKey(bytes.fromhex(public_key_hex))
    
    # Get the address using the library method (this is the correct way)
    address = public_key.to_address()
    
    # Convert to checksum address
    checksum_address = to_checksum_address(address)
    
    return checksum_address

def main():
    # The public key from the README
    public_key = "03a53e97510bc4118b829adcb87b29cadc9d29c36dd204c9bd63a2a526fe3a93a1"
    
    print("=== 以太坊地址计算 ===")
    print(f"输入的公钥: {public_key}")
    print()
    
    try:
        # Calculate Ethereum address
        ethereum_address = calculate_ethereum_address_correct(public_key)
        
        print("计算结果:")
        print(f"以太坊地址: {ethereum_address}")
        print()
        
        # Also show without checksum for reference
        address_without_checksum = ethereum_address.lower()
        print(f"地址（小写）: {address_without_checksum}")
        
        print("\n=== 技术说明 ===")
        print("1. 公钥格式: secp256k1 压缩格式 (以 03 开头)")
        print("2. 计算过程:")
        print("   - 将压缩公钥转换为未压缩格式")
        print("   - 对未压缩公钥的 x,y 坐标进行 Keccak-256 哈希")
        print("   - 取哈希值的最后 20 字节作为地址")
        print("   - 应用 EIP-55 校验和格式")
        print("3. 地址格式: 符合以太坊标准 (0x + 40 个十六进制字符)")
        
    except Exception as e:
        print(f"计算错误: {e}")

if __name__ == "__main__":
    main() 