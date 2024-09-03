import {useState} from "preact/hooks";
import {Input} from "@/components/ui/input.tsx";
import {Button} from "@/components/ui/button.tsx";
import {Eye, EyeOff} from "lucide-react";

interface PasswordInputParams {
    placeholder: string,
    setValue: (value: string) => void
}

export const PasswordInput = ({placeholder, setValue}: PasswordInputParams) => {
    let [shouldShowPassword, showPassword] = useState(false);
    return <div className="flex w-full items-center space-x-2">
        <Input
            onChange={(event) => {
                setValue((event.currentTarget as HTMLInputElement).value)
            }}
            type={(shouldShowPassword) ?
                "text" : "password"} placeholder={placeholder}/>
        <Button onClick={() => {
            showPassword((prev) => !prev);
        }}>{
            (shouldShowPassword) ?
                <EyeOff/> : <Eye/>
        }</Button>
    </div>
}