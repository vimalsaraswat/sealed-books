import { secp256k1 } from "@noble/curves/secp256k1.js";
import { keccak_256 } from "@noble/hashes/sha3.js";

export interface ApproverPersona {
  id: string; // matches user.id in db (usr_alice, usr_bob, etc.)
  name: string;
  role: string;
  email: string;
  token: string;
  privateKeyHex: string;
}

// Four deterministic personas for multi-tenant and separation of duties testing
export const DEMO_APPROVERS: ApproverPersona[] = [
  {
    id: "usr_alice",
    name: "Alice Vance",
    role: "controller",
    email: "alice@acmetrading.com",
    token: "token_alice",
    privateKeyHex:
      "1111111111111111111111111111111111111111111111111111111111111111",
  },
  {
    id: "usr_bob",
    name: "Bob Stone",
    role: "auditor",
    email: "bstone@auditfirm.com",
    token: "token_bob",
    privateKeyHex:
      "2222222222222222222222222222222222222222222222222222222222222222",
  },
  {
    id: "usr_charlie",
    name: "Charlie Davis",
    role: "staff",
    email: "charlie@acmetrading.com",
    token: "token_charlie",
    privateKeyHex:
      "3333333333333333333333333333333333333333333333333333333333333333",
  },
  {
    id: "usr_diana",
    name: "Diana Vance",
    role: "owner",
    email: "diana@acmetrading.com",
    token: "token_diana",
    privateKeyHex:
      "4444444444444444444444444444444444444444444444444444444444444444",
  },
];


/**
 * Retrieves or generates an ECDSA secp256k1 private key for a user.
 * Seed test users use deterministic keys; real new users generate and persist keys in local storage.
 */
export function getUserSigningKey(userId: string): string {
  const seedKeyMap: Record<string, string> = {
    usr_alice: "1111111111111111111111111111111111111111111111111111111111111111",
    usr_bob: "2222222222222222222222222222222222222222222222222222222222222222",
    usr_charlie: "3333333333333333333333333333333333333333333333333333333333333333",
    usr_diana: "4444444444444444444444444444444444444444444444444444444444444444",
  };

  if (seedKeyMap[userId]) {
    return seedKeyMap[userId];
  }

  if (typeof window !== "undefined" && window.localStorage) {
    const storageKey = `sb_user_key_${userId}`;
    const existing = localStorage.getItem(storageKey);
    if (existing) {
      return existing;
    }
    const randomPrivKey = secp256k1.utils.randomSecretKey();
    const hexKey = bytesToHex(randomPrivKey);
    localStorage.setItem(storageKey, hexKey);
    return hexKey;
  }

  return "1111111111111111111111111111111111111111111111111111111111111111";
}

export function getApproverById(id: string): ApproverPersona | undefined {
  return DEMO_APPROVERS.find(
    (p) =>
      p.id === id ||
      (id === "approver-1" && p.id === "usr_alice") ||
      (id === "approver-2" && p.id === "usr_bob"),
  );
}

export function getApproverDetails(persona: ApproverPersona) {
  const privKey = hexToBytes(persona.privateKeyHex);
  const pubKeyBytes = secp256k1.getPublicKey(privKey, true); // 33-byte compressed
  const pubkeyHex = bytesToHex(pubKeyBytes);

  // Compute Ethereum address from uncompressed public key (skip prefix byte)
  const uncompressedPubKey = secp256k1.getPublicKey(privKey, false).slice(1);
  const hash = keccak_256(uncompressedPubKey);
  const address = "0x" + bytesToHex(hash.slice(-20));

  return { pubkeyHex, address };
}

export function signStatementHash(
  statementHashHex: string,
  privateKeyHex: string,
): { signatureHex: string; pubkeyHex: string; address: string } {
  const cleanHash = statementHashHex.startsWith("0x")
    ? statementHashHex.slice(2)
    : statementHashHex;
  const hashBytes = hexToBytes(cleanHash);
  const privKeyBytes = hexToBytes(privateKeyHex);

  // Sign prehash without re-hashing
  const signatureBytes = secp256k1.sign(hashBytes, privKeyBytes, {
    prehash: false,
  });
  const signatureHex = bytesToHex(signatureBytes);

  const pubKeyBytes = secp256k1.getPublicKey(privKeyBytes, true);
  const pubkeyHex = bytesToHex(pubKeyBytes);

  const uncompressed = secp256k1.getPublicKey(privKeyBytes, false).slice(1);
  const ethHash = keccak_256(uncompressed);
  const address = "0x" + bytesToHex(ethHash.slice(-20));

  return {
    signatureHex,
    pubkeyHex,
    address,
  };
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.length % 2 !== 0 ? "0" + hex : hex;
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
