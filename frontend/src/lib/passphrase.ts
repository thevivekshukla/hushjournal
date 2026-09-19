export const MIN_PASSPHRASE_LEN = 8;

export function minPassphraseLengthError(field: string) {
	return `${field} must be at least ${MIN_PASSPHRASE_LEN} characters.`;
}

export type PassphraseScore = 0 | 1 | 2 | 3 | 4;

const LABELS = ['', 'Weak', 'Fair', 'Good', 'Strong'] as const;

const COMMON = new Set([
	'password',
	'passphrase',
	'password1',
	'passphrase1',
	'12345678',
	'123456789',
	'qwerty',
	'qwertyuiop',
	'letmein',
	'journal',
	'e2ejournal'
]);

function charsetSize(value: string) {
	let size = 0;
	if (/[a-z]/.test(value)) size += 26;
	if (/[A-Z]/.test(value)) size += 26;
	if (/\d/.test(value)) size += 10;
	if (/[^A-Za-z0-9]/.test(value)) size += 33;
	return Math.max(size, 1);
}

export function passphraseStrength(value: string): { score: PassphraseScore; label: string } {
	if (!value) return { score: 0, label: '' };

	const unique = new Set(value).size;
	const entropy = value.length * Math.log2(charsetSize(value)) * (unique / value.length);

	let score: PassphraseScore = 1;
	if (value.length >= 8 && entropy >= 28) score = 2;
	if (value.length >= 12 && entropy >= 40) score = 3;
	if (value.length >= 16 && entropy >= 52) score = 4;

	if (unique <= 2 || /^(.)\1+$/.test(value) || COMMON.has(value.toLowerCase())) score = 1;

	return { score, label: LABELS[score] };
}
