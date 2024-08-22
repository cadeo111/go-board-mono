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