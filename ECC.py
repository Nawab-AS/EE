import random

def mod_inverse(n, p):
    """Calculates the modular multiplicative inverse using Fermat's Little Theorem."""
    return pow(n, p - 2, p)

class WeierstrassCurve:
    def __init__(self, a, b, p, g_x, g_y):
        self.a = a
        self.b = b
        self.p = p
        self.G = (g_x, g_y)

    def point_add(self, P1, P2):
        """Adds two points on the curve. Handles identity and doubling."""
        if P1 is None: return P2
        if P2 is None: return P1
        x1, y1 = P1
        x2, y2 = P2
        if x1 == x2 and y1 != y2: return None 
        if x1 == x2:
            m = (3 * x1**2 + self.a) * mod_inverse(2 * y1, self.p)
        else:
            m = (y2 - y1) * mod_inverse(x2 - x1, self.p)
        x3 = (m**2 - x1 - x2) % self.p
        y3 = (m * (x1 - x3) - y1) % self.p
        return (x3, y3)

    def multiply(self, P, scalar):
        """Scalar Multiplication using Double-and-Add (Logarithmic time)."""
        result = None
        addend = P
        while scalar:
            if scalar & 1:
                result = self.point_add(result, addend)
            addend = self.point_add(addend, addend)
            scalar >>= 1
        return result

# --- THE ECDH "KEY AGREEMENT" SIMULATION ---

# Setup a toy Weierstrass Curve: y^2 = x^3 + 2x + 2 mod 997
curve = WeierstrassCurve(a=2, b=2, p=997, g_x=2, g_y=505)

print("--- INITIALIZATION: BOTH PARTIES GENERATE THEIR IDENTITIES ---")
# Bob and Alice both generate their own private/public key pairs
# Unlike RSA, Bob doesn't 'own' the secret; both contribute.

# Bob's Keys
p_bob_priv = random.randint(100, 900)
p_bob_pub = curve.multiply(curve.G, p_bob_priv)

# Alice's Keys
p_alice_priv = random.randint(100, 900)
p_alice_pub = curve.multiply(curve.G, p_alice_priv)

print(f"[Bob] My Public Point is: {p_bob_pub}")
print(f"[Bob] My Private Key is: {p_bob_priv} (I keep this hidden!)")
print(f"[Alice] My Public Point is: {p_alice_pub}")
print(f"[Alice] My Private Key is: {p_alice_priv} (I keep this hidden!)\n")


print("--- STEP 1: ALICE COMPUTES THE SHARED SECRET ---")
# Alice takes BOB'S public point and multiplies it by HER private key
# s = d_alice * P_bob
alice_shared_point = curve.multiply(p_bob_pub, p_alice_priv)
alice_secret = alice_shared_point[0] # We use the X-coordinate as the secret
print(f"[Alice] I've computed the shared secret from Bob's point: {alice_secret}\n")


print("--- STEP 2: BOB COMPUTES THE SHARED SECRET ---")
# Bob takes ALICE'S public point and multiplies it by HIS private key
# s = d_bob * P_alice
bob_shared_point = curve.multiply(p_alice_pub, p_bob_priv)
bob_secret = bob_shared_point[0]
print(f"[Bob] I've computed the shared secret from Alice's point: {bob_secret}\n")


print("--- STEP 3: ESTABLISHING SECURE COMMUNICATION ---")
if alice_secret == bob_secret:
    print(f"[Success] Alice and Bob now have a shared secret: {alice_secret}")
    
    # Now they switch to Symmetric Encryption (XOR for demo)
    message = "EC is 10x more efficient than RSA!"
    print(f"[Alice] Original Message: {message}")

    # ENCRYPT
    encrypted_bytes = [ord(c) ^ alice_secret for c in message]
    print(f"[Alice] Sending encrypted list of ints: {encrypted_bytes}")
    
    # DECRYPT
    decrypted_msg = "".join(chr(b ^ bob_secret) for b in encrypted_bytes)
    print(f"[Bob] Decrypted chat message: {decrypted_msg}")
