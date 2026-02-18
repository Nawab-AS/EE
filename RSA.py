import random

def extended_gcd(a, b):
    if a == 0: return b, 0, 1
    gcd, x1, y1 = extended_gcd(b % a, a)
    x = y1 - (b // a) * x1
    y = x1
    return gcd, x, y

def get_mod_inverse(e, phi):
    gcd, x, y = extended_gcd(e, phi)
    return x % phi

# --- THE RSA "KEY TRANSPORT" SIMULATION ---

print("--- INITIALIZATION: BOB GENERATES HIS IDENTITY ---")
# Bob picks two primes and generates his public/private key pair
p_bob, q_bob = 61, 53
n_bob = p_bob * q_bob
phi_bob = (p_bob - 1) * (q_bob - 1)
e_bob = 17
d_bob = get_mod_inverse(e_bob, phi_bob)

print(f"[Bob] My Public Key is: (e={e_bob}, n={n_bob})")
print(f"[Bob] My Private Key is: {d_bob} (I keep this hidden!)\n")


print("--- STEP 1: ALICE PREPARES A SECRET ---")
# Alice wants to start an encrypted chat. She generates a random session key.
# In a real app, this would be a 256-bit AES key.
alice_session_key = random.randint(2, n_bob - 1) 
print(f"[Alice] I've generated a random session key: {alice_session_key}")

# Alice encrypts her session key using BOB'S public key
encrypted_secret = pow(alice_session_key, e_bob, n_bob)
print(f"[Alice] I'm sending this encrypted blob to Bob: {encrypted_secret}\n")


print("--- STEP 2: BOB RECEIVES AND DECRYPTS ---")
# Bob receives the blob. Only he can open it because only he has d_bob.
bob_recovered_key = pow(encrypted_secret, d_bob, n_bob)
print(f"[Bob] I received the blob. Decrypting... recovered key: {bob_recovered_key}\n")


print("--- STEP 3: ESTABLISHING SECURE COMMUNICATION ---")
if alice_session_key == bob_recovered_key:
    print(f"[Success] Alice and Bob share secret: {alice_session_key}")
    
    message = "Hello Bob! RSA is cool."
    print(f"[Alice] Original Message: {message}")

    # 1. ENCRYPT: Work with integers, not characters
    # We create a list of numbers (the 'ciphertext')
    encrypted_bytes = [ord(c) ^ alice_session_key for c in message]
    
    # We print the raw numbers so the terminal doesn't crash
    print(f"[Alice] Sending encrypted list of ints: {encrypted_bytes}")
    
    # 2. DECRYPT: Bob uses his recovered key on the list of ints
    decrypted_chars = [chr(b ^ bob_recovered_key) for b in encrypted_bytes]
    decrypted_msg = "".join(decrypted_chars)
    
    print(f"[Bob] Decrypted chat message: {decrypted_msg}")