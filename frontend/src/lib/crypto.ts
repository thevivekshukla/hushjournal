import { gcmsiv } from '@noble/ciphers/aes.js';
import { randomBytes } from '@noble/ciphers/utils.js';
import { argon2idAsync } from '@noble/hashes/argon2.js';

const VERSION = 1;
const NONCE_LEN = 12;
const SALT_LEN = 16;
const DEK_LEN = 32;
const ARGON2 = { t: 3, m: 65_536, p: 1, dkLen: DEK_LEN, asyncTick: 16 } as const;

export type Purpose = 'dek' | 'notebook.name' | 'entry.title' | 'entry.content';

export class CryptoError extends Error {
	constructor(message = 'Could not decrypt') {
		super(message);
		this.name = 'CryptoError';
	}
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function aad(purpose: Purpose, id?: string): Uint8Array {
	// v1 is not tied to a row. Leave that string unchanged so existing blobs still open.
	// v2 includes the notebook or entry id so a swapped blob fails authentication.
	if (id) return encoder.encode(`hushjournal:v2:${purpose}:${id}`);
	return encoder.encode(`hushjournal:v1:${purpose}`);
}

async function deriveKek(passphrase: string, salt: Uint8Array) {
	return argon2idAsync(encoder.encode(passphrase), salt, ARGON2);
}

export function encryptBytes(
	key: Uint8Array,
	plaintext: Uint8Array,
	purpose: Purpose,
	id?: string
): Uint8Array {
	const nonce = randomBytes(NONCE_LEN);
	const ciphertext = gcmsiv(key, nonce, aad(purpose, id)).encrypt(plaintext);
	const out = new Uint8Array(1 + nonce.length + ciphertext.length);
	out[0] = VERSION;
	out.set(nonce, 1);
	out.set(ciphertext, 1 + nonce.length);
	return out;
}

export function decryptBytes(
	key: Uint8Array,
	blob: Uint8Array,
	purpose: Purpose,
	id?: string
): Uint8Array {
	if (blob.length < 1 + NONCE_LEN + 16 || blob[0] !== VERSION) {
		throw new CryptoError();
	}
	const nonce = blob.subarray(1, 1 + NONCE_LEN);
	const ciphertext = blob.subarray(1 + NONCE_LEN);
	try {
		return gcmsiv(key, nonce, aad(purpose, id)).decrypt(ciphertext);
	} catch {
		throw new CryptoError();
	}
}

export function encryptText(
	key: Uint8Array,
	plaintext: string,
	purpose: Purpose,
	id?: string
): Uint8Array {
	return encryptBytes(key, encoder.encode(plaintext), purpose, id);
}

export function decryptText(
	key: Uint8Array,
	blob: Uint8Array,
	purpose: Purpose,
	id: string
): { text: string; legacy: boolean } {
	try {
		return { text: decoder.decode(decryptBytes(key, blob, purpose, id)), legacy: false };
	} catch (error) {
		if (!(error instanceof CryptoError)) throw error;
	}
	return { text: decoder.decode(decryptBytes(key, blob, purpose)), legacy: true };
}

export async function wrapDek(passphrase: string, dek: Uint8Array) {
	if (dek.length !== DEK_LEN) throw new CryptoError();
	const keySalt = randomBytes(SALT_LEN);
	const kek = await deriveKek(passphrase, keySalt);
	try {
		return { keySalt, encryptedDek: encryptBytes(kek, dek, 'dek') };
	} finally {
		kek.fill(0);
	}
}

export async function createJournalSecrets(passphrase: string) {
	const dek = randomBytes(DEK_LEN);
	const wrapped = await wrapDek(passphrase, dek);
	return { dek, ...wrapped };
}

export function equalBytes(a: Uint8Array, b: Uint8Array) {
	if (a.length !== b.length) return false;
	let diff = 0;
	for (let i = 0; i < a.length; i++) diff |= a[i] ^ b[i];
	return diff === 0;
}

export async function unlockDek(
	passphrase: string,
	keySalt: Uint8Array,
	encryptedDek: Uint8Array
): Promise<Uint8Array> {
	const kek = await deriveKek(passphrase, keySalt);
	try {
		const dek = decryptBytes(kek, encryptedDek, 'dek');
		if (dek.length !== DEK_LEN) throw new CryptoError('Wrong passphrase');
		return dek;
	} catch {
		throw new CryptoError('Wrong passphrase');
	} finally {
		kek.fill(0);
	}
}

export function bytesToBase64(bytes: Uint8Array): string {
	const chunks: string[] = [];
	const size = 0x8000;
	for (let i = 0; i < bytes.length; i += size) {
		chunks.push(String.fromCharCode(...bytes.subarray(i, i + size)));
	}
	return btoa(chunks.join(''));
}

export function base64ToBytes(value: string): Uint8Array {
	const binary = atob(value.trim());
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

export function zeroKey(key: Uint8Array | null) {
	key?.fill(0);
}
