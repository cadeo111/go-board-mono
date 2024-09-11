import {WifiCredentialsCard} from "@/components/custom/WifiCredentialsCard.tsx";
import {GoOnlineCredentialsCard} from "@/components/custom/GoOnlineCredentialsCard.tsx";


import {ComponentChild} from "preact";
import {useState} from "preact/hooks";
import {useEffect} from "preact/compat";
import {generateMaskedPassword, I_GenericResponse} from "@/lib/utils.ts";

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


interface I_WifiStatus {
    connected: boolean,
    ssid: string,
    first_letter_of_password: string,
    length_of_password: number,
}

const getWifiInfo = async (): Promise<I_WifiStatus | null> => {
    let response = await fetch("/wifi-status")

    let responseJson = await response.json() as I_GenericResponse<I_WifiStatus, any>;
    if (responseJson.is_ok) {
        return responseJson.value
    } else {
        alert(`ERROR UPDATING WIFI CREDS see console`)
        console.error(responseJson.value)
        return null;
    }

}


export function App() {

    // example query string => /?w_id=randomssid&w_pfc=s&w_pn=6&og_un=cade&og_pfc=R&og_pn=20&w_c=c


    const [wifiStatus, setWifiStatus] = useState<null | I_WifiStatus>(null);
    useEffect(() => {
        getWifiInfo().then((info) => {
            setWifiStatus(info)
        })
    }, [])
    let [isWifiLoading, setWifiLoading] = useState(false);
    const saveWifiCredentials = async (ssid: string, password: string) => {
        setWifiLoading(true);

        const response = await fetch("/save-wifi-credentials", {
            method: "POST",
            body: JSON.stringify({ssid, password}),
        });
        if (!response.ok) {
            // TODO: better request error handling (Popup?)
            alert("ERROR UPDATING WIFI CREDS REQ")
            return
        }
    }


    return (
        <StyleWrapper>
            <WifiCredentialsCard
                initialSSID={wifiStatus?.ssid ?? null}
                hiddenPassword={generateMaskedPassword(wifiStatus?.first_letter_of_password ?? null, wifiStatus?.length_of_password ?? null)}
                onSaveWifiCredentials={async ({ssid, password}) => {
                    await saveWifiCredentials(ssid, password)
                }}
                loading={isWifiLoading}
                connected={wifiStatus?.connected ?? false}/>

            <GoOnlineCredentialsCard/>


        </StyleWrapper>
    )
}
