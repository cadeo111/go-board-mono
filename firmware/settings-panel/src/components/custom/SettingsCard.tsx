import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "@/components/ui/card.tsx";
import {Badge} from "@/components/ui/badge.tsx";
import {Button} from "@/components/ui/button.tsx";
import {LoaderPinwheel} from "lucide-react";
import {PropsWithChildren} from "preact/compat";
import * as preact from "preact";

interface SettingsCardProps {
    loading: boolean;
    error: boolean;
    title: string;
    description: string;
    noErrorBadgeText: string;
    errorBadgeText: string;
    onSave?: () => void;
    buttonElement?: preact.ComponentChild;
}


export const SettingsCard = ({
                                 loading,
                                 error,
                                 title,
                                 description,
                                 noErrorBadgeText,
                                 errorBadgeText,
                                 onSave,
                                 buttonElement,
                                 children
                             }: PropsWithChildren<SettingsCardProps>) => {

    const btn = (onSave !== undefined) ?
        <Button onClick={onSave}>Save</Button> :
        buttonElement


    return <Card className="relative">
        {loading && <div className="grid place-items-center w-full h-full absolute top-0 left-0 right-0 bottom-0 bg-blue-200/50">
            <LoaderPinwheel strokeWidth={1} size={100} className="animate-spin-slow opacity-100 stroke-blue-300"/>
        </div>}
        {(!loading && error) && <div
            className="w-full h-full absolute top-0 left-0 right-0 bottom-0 pointer-events-none animate-pulse border-error border bg-error/5"></div>}
        <CardHeader>
            <CardTitle>{title}</CardTitle>
            <CardDescription>
                {description}
            </CardDescription>
        </CardHeader>
        <CardContent>

            {!error ? <Badge className="border-success text-success mb-3" variant="outline">{noErrorBadgeText}</Badge> :
                <Badge className=" border-error text-error mb-3" variant="outline">{errorBadgeText}</Badge>}
            <div>
                {children}
            </div>
        </CardContent>
        <CardFooter className="border-t px-6 py-4">
            {btn}
        </CardFooter>
    </Card>
}