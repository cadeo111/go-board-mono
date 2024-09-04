import {Button} from "@/components/ui/button"
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,} from "@/components/ui/card"
import {Label} from "@/components/ui/label"
import {Badge} from "@/components/ui/badge.tsx";
import {Combobox} from "@/components/custom/Combobox.tsx";
import {WifiCredentialsCard} from "@/components/custom/WifiCredentialsCard.tsx";
import {GoOnlineCredentialsCard} from "@/components/custom/GoOnlineCredentialsCard.tsx";


import {ComponentChild} from "preact";
import {useState} from "preact/hooks";
import {useEffect} from "preact/compat";

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

function generateMaskedPassword(initialPasswordChar: string | null, initialPasswordNum: number | null): null | string {
    const passwordNum = initialPasswordNum ?? 0;

    if (passwordNum == 0) return null;

    if (initialPasswordChar == null) return "*".repeat(passwordNum);

    return initialPasswordChar[0] + "*".repeat(passwordNum - 1);
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
    if (!response.ok) {
        alert("ERROR UPDATING WIFI CREDS REQ")
        return null;
    }
    return await response.json() as I_WifiStatus;
}


export function App() {

    // example query string => /?w_id=randomssid&w_pfc=s&w_pn=6&og_un=cade&og_pfc=R&og_pn=20&w_c=c


    const [wifiStatus, setWifiStatus] = useState<null | I_WifiStatus>(null);

    useEffect(() => {
        getWifiInfo().then((info) => {
            setWifiStatus(info)
        })
    }, [])


    const qs = new URLSearchParams(location.search);


    let [isWifiLoading, setWifiLoading] = useState(false);

    const initialOnlineGoUsername = qs.get("og_un") ?? "";
    const initialOnlineGoPasswordChar = qs.get("og_pfc") ?? "";
    const initialOnlineGoPasswordCharNum = qs.get("og_pn");


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
            <GoOnlineCredentialsCard
                authorized={false}
                initialUsername={initialOnlineGoUsername}
                hiddenPassword={generateMaskedPassword(initialOnlineGoPasswordChar, Number(initialOnlineGoPasswordCharNum))}
                onSaveCredentials={(username, password) => {
                    console.log(username, password)
                }}
                loading={false}/>

            <Card id="go_board_settings">
                <CardHeader>
                    <CardTitle>Go Game Settings</CardTitle>
                    <CardDescription>
                        Used to set up game information
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <Badge className="border-success text-success mb-3" variant="outline">Verified</Badge>
                    <form className="grid gap-4">
                        <div>
                            <Label>Select Game</Label>
                            <div className="flex w-full items-center">
                                <Combobox<number>
                                    options={[
                                        {
                                            label: "[5/12] cadeo111 (9x9)", value: 0,
                                        },
                                        {
                                            label: "[4/16] cd113 (9x9)", value: 1,
                                        },
                                    ]}
                                    placeholderSelect={"Select game..."}
                                    placeholderSearch={"Search for game..."}
                                />
                            </div>
                        </div>
                        <div><Label>Your Color</Label>
                            <div className="flex w-full items-center">
                                <Combobox<number>
                                    options={[
                                        {
                                            label: "Blue", value: 0,
                                        },
                                        {
                                            label: "Green", value: 1,
                                        },
                                        {
                                            label: "Red", value: 2,
                                        },
                                    ]}
                                    placeholderSelect={"Select color..."}
                                    placeholderSearch={"Search for color..."}
                                />
                            </div>
                        </div>
                        <div>
                            <Label>Other Player Color</Label>
                            <div className="flex w-full items-center">
                                <Combobox<number>
                                    options={[
                                        {
                                            label: "Blue", value: 0,
                                        },
                                        {
                                            label: "Green", value: 1,
                                        },
                                        {
                                            label: "Red", value: 2,
                                        },
                                    ]}
                                    placeholderSelect={"Select color..."}
                                    placeholderSearch={"Search for color..."}
                                />
                            </div>
                        </div>

                    </form>
                </CardContent>
                <CardFooter className="border-t px-6 py-4">
                    <Button>Save</Button>
                </CardFooter>
            </Card>

        </StyleWrapper>
    )
}
