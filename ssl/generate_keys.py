import os
from datetime import datetime, timedelta
from cryptography import x509
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.backends import default_backend
from cryptography.x509.oid import NameOID

def generate_jwt_keys(private_key_path="jwt_private_key.pem", public_key_path="jwt_public_key.pem"):
    """
    Generates an RSA private and public key pair in PEM format for JWTs.

    Args:
        private_key_path (str): The file path to save the JWT private key.
        public_key_path (str): The file path to save the JWT public key.
    """
    try:
        # Generate RSA private key for JWT
        print("Generating RSA private key for JWT...")
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048, # 2048 bits is a common and secure key size
            backend=default_backend()
        )
        print("JWT private key generated.")

        # Serialize private key to PEM format
        # Use NoEncryption for simplicity; for production, consider using a strong password
        private_pem = private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )

        # Save private key to file
        with open(private_key_path, "wb") as f:
            f.write(private_pem)
        print(f"JWT private key saved to {private_key_path}")

        # Get the public key from the private key
        public_key = private_key.public_key()

        # Serialize public key to PEM format
        public_pem = public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )

        # Save public key to file
        with open(public_key_path, "wb") as f:
            f.write(public_pem)
        print(f"JWT public key saved to {public_key_path}")

        print("\nJWT key generation complete.")
        print(f"Use '{private_key_path}' for JWT encoding (signing).")
        print(f"Use '{public_key_path}' for JWT decoding (verification).")

    except Exception as e:
        print(f"An error occurred during JWT key generation: {e}")

def generate_tls_keys_and_certificate(
    private_key_path="tls_private_key.pem",
    certificate_path="tls_certificate.pem",
    common_name="localhost",
    days_valid=365
):
    """
    Generates an RSA private key and a self-signed X.509 certificate in PEM format for TLS/HTTPS.

    Args:
        private_key_path (str): The file path to save the TLS private key.
        certificate_path (str): The file path to save the TLS certificate.
        common_name (str): The Common Name for the certificate (e.g., your domain or "localhost").
        days_valid (int): The number of days the certificate will be valid.
    """
    try:
        # Generate RSA private key for TLS
        print("\nGenerating RSA private key for TLS...")
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048, # 2048 bits is a common and secure key size
            backend=default_backend()
        )
        print("TLS private key generated.")

        # Serialize TLS private key to PEM format
        private_pem = private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )

        # Save TLS private key to file
        with open(private_key_path, "wb") as f:
            f.write(private_pem)
        print(f"TLS private key saved to {private_key_path}")

        # Generate a self-signed certificate
        print("Generating self-signed TLS certificate...")
        subject = issuer = x509.Name([
            x509.NameAttribute(NameOID.COUNTRY_NAME, u"US"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, u"CA"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, u"San Francisco"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, u"MyOrg"),
            x509.NameAttribute(NameOID.COMMON_NAME, common_name),
        ])

        certificate = (
            x509.CertificateBuilder()
            .subject_name(subject)
            .issuer_name(issuer)
            .public_key(private_key.public_key())
            .serial_number(x509.random_serial_number())
            .not_valid_before(datetime.utcnow())
            .not_valid_after(datetime.utcnow() + timedelta(days=days_valid))
            .add_extension(x509.SubjectAlternativeName([x509.DNSName(common_name)]), critical=False,)
            .sign(private_key, hashes.SHA256(), default_backend())
        )

        # Serialize certificate to PEM format
        certificate_pem = certificate.public_bytes(serialization.Encoding.PEM)

        # Save certificate to file
        with open(certificate_path, "wb") as f:
            f.write(certificate_pem)
        print(f"TLS certificate saved to {certificate_path}")

        print("\nTLS key and certificate generation complete.")
        print(f"Use '{private_key_path}' as your TLS private key.")
        print(f"Use '{certificate_path}' as your TLS certificate.")

    except Exception as e:
        print(f"An error occurred during TLS key and certificate generation: {e}")

if __name__ == "__main__":
    # Generate JWT keys
    generate_jwt_keys()

    print("\n" + "="*50 + "\n") # Separator for clarity

    # Generate TLS keys and certificate
    # You can change 'localhost' to your domain name if needed
    generate_tls_keys_and_certificate(common_name="192.168.1.63")

