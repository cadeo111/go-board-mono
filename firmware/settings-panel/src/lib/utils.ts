import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"
import {ComponentPropsWithRef, ForwardRefExoticComponent, RefAttributes} from "preact/compat";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export type ElementRef<
    C extends ForwardRefExoticComponent<any>
> =
// need to check first if `ref` is a valid prop for ts@3.0
// otherwise it will infer `{}` instead of `never`
    "ref" extends keyof ComponentPropsWithRef<C>
        ? NonNullable<ComponentPropsWithRef<C>["ref"]> extends RefAttributes<
                infer Instance
            >["ref"] ? Instance
            : never
        : never;

export function generateMaskedPassword(initialPasswordChar: string | null, initialPasswordNum: number | null): null | string {
    const passwordNum = initialPasswordNum ?? 0;

    if (passwordNum == 0) return null;

    if (initialPasswordChar == null) return "*".repeat(passwordNum);

    return initialPasswordChar[0] + "*".repeat(passwordNum - 1);
}



export interface I_GenericOk<OK> {
    "is_ok": true,
    "value": OK
}

export interface I_GenericError<ERROR> {
    "is_ok": false,
    "value": ERROR
}

export type I_GenericResponse<OK, ERROR> = I_GenericOk<OK> | I_GenericError<ERROR>
