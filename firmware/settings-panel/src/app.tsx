import {WifiCredentialsCard} from "@/components/custom/WifiCredentialsCard.tsx";
import {GoOnlineCredentialsCard} from "@/components/custom/GoOnlineCredentialsCard.tsx";


import {ComponentChild} from "preact";


const StyleWrapper = ({children}: { children: ComponentChild }) => {
    return <div className="flex min-h-screen w-full flex-col">
        <header className="sticky  z-20  top-0 flex h-16 items-center gap-4 border-b bg-background px-4 md:px-6">
            <div className="flex w-full items-center gap-4 md:ml-auto md:gap-2 lg:gap-4">
                <h1 className="text-3xl font-semibold"> Go Board Settings</h1>
            </div>
        </header>
        <main className="flex min-h-[calc(100vh_-_theme(spacing.16))] flex-1 flex-col gap-4 bg-muted/40 p-4 md:gap-8 md:p-10">
            <div className="mx-auto flex w-full max-w-6xl items-start justify-center gap-6">
                <div className="grid gap-6 md:w-6/12">
                    {children}
                </div>
            </div>
        </main>
    </div>;
}


// TODO: rename all interfaces as I_*





export function App() {

    // example query string => /?w_id=randomssid&w_pfc=s&w_pn=6&og_un=cade&og_pfc=R&og_pn=20&w_c=c





    return (
        <StyleWrapper>
            <WifiCredentialsCard/>
            <GoOnlineCredentialsCard/>


        </StyleWrapper>
    )
}
