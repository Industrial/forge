import { Schema } from "effect";

/** Non-empty trimmed string (e.g. for email, names). */
export const nonEmptyTrimmedString = Schema.NonEmptyTrimmedString;

/** Email: non-empty trimmed string with basic email pattern. */
export const emailSchema = Schema.NonEmptyTrimmedString.pipe(
	Schema.pattern(/^[^\s@]+@[^\s@]+\.[^\s@]+$/, {
		message: () => "Enter a valid email address",
	}),
);

/** Password: min 8 characters. */
export const passwordMin8Schema = Schema.NonEmptyTrimmedString.pipe(
	Schema.minLength(8, { message: () => "Password must be at least 8 characters" }),
);

/** Single organization ID (non-empty string). */
export const orgIdSchema = Schema.NonEmptyTrimmedString;

/** At least one role ID. Validated in form schema via filter or in submit. */
export const roleIdsSchema = Schema.Array(Schema.String);

/** Optional string for optional form fields. */
export const optionalTrimmedString = Schema.optional(Schema.String);
