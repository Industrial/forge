import { Schema } from "effect";
import type { FieldErrors, FieldValues, Resolver } from "react-hook-form";

function setNested(
	obj: Record<string, unknown>,
	path: readonly string[],
	value: { message: string },
): void {
	let current: Record<string, unknown> = obj;
	for (let i = 0; i < path.length - 1; i++) {
		const key = path[i];
		if (!(key in current) || typeof current[key] !== "object" || current[key] === null) {
			current[key] = {};
		}
		current = current[key] as Record<string, unknown>;
	}
	const lastKey = path[path.length - 1];
	current[lastKey] = value;
}

/**
 * Build a React Hook Form resolver from an Effect Schema (with no required context).
 * Validates form values with Schema.decodeUnknownEither and maps parse errors
 * to RHF's FieldErrors (nested by path).
 */
export function effectSchemaResolver<A extends FieldValues>(
	schema: Schema.Schema<A, unknown, never>,
): Resolver<A> {
	return (values) => {
		const either = Schema.decodeUnknownEither(schema)(values as unknown);
		if (either._tag === "Right") {
			return { values: either.right, errors: {} };
		}
		const parseError = either.left;
		const errors: Record<string, unknown> = {};
		const path =
			typeof parseError === "object" &&
			parseError !== null &&
			"path" in parseError &&
			Array.isArray((parseError as { path: unknown }).path)
				? ((parseError as { path: string[] }).path as string[])
				: [];
		const message =
			typeof parseError === "object" && parseError !== null && "message" in parseError
				? String((parseError as { message: unknown }).message)
				: "Validation failed";
		if (path.length > 0) {
			setNested(errors, path, { message });
		} else {
			errors.root = { message };
		}
		return { values: {} as Record<string, never>, errors: errors as FieldErrors<A> };
	};
}
