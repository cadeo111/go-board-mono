import {Button} from "@/components/ui/button"
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,} from "@/components/ui/card"
import {Label} from "@/components/ui/label"
import {Badge} from "@/components/ui/badge.tsx";
import {useState} from "preact/hooks";
import {Combobox} from "@/components/custom/Combobox.tsx";
import {WifiCredentialsCard} from "@/components/custom/WifiCredentialsCard.tsx";
import {GoOnlineCredentialsCard} from "@/components/custom/GoOnlineCredentialsCard.tsx";
import {
    AlertDialog, AlertDialogAction, AlertDialogCancel,
    AlertDialogContent, AlertDialogDescription, AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger
} from "@/components/ui/alert-dialog.tsx";


export function App() {
    let qs = new URLSearchParams(location.search);
    let initialWifiSSID = qs.get("wLid") ?? "";
    let initialOnlineGoUsername = qs.get("ogLun") ?? "";

    const [wifiSSID, setWifiSSID] = useState(initialWifiSSID)
    const [wifiPassword, setWifiPassword] = useState("")

    const [onlineGoUsername, setOnlineGoUsername] = useState(initialOnlineGoUsername)
    const [onlineGoPassword, setOnlineGoPassword] = useState("")


    const saveWifiCredentials = () => {
        let ssid = wifiSSID;
        let password = wifiPassword;
        console.log(ssid, password)
    }
    const saveOnlineGoCredentials = () => {
        let username = onlineGoUsername;
        let password = onlineGoPassword;
        console.log(username, password)
    }


    return (
        <div className="flex min-h-screen w-full flex-col">
            <header className="sticky  z-20  top-0 flex h-16 items-center gap-4 border-b bg-background px-4 md:px-6">
                <div className="flex w-full items-center gap-4 md:ml-auto md:gap-2 lg:gap-4">
                    <h1 className="text-3xl font-semibold"> Go Board Settings</h1>
                </div>
            </header>
            <main className="flex min-h-[calc(100vh_-_theme(spacing.16))] flex-1 flex-col gap-4 bg-muted/40 p-4 md:gap-8 md:p-10">
                <div className="mx-auto flex w-full max-w-6xl items-start justify-center gap-6">
                    <div className="grid gap-6 md:w-6/12">
                        <WifiCredentialsCard
                            initialSSID={"random_ssid"}
                            hiddenPassword={"H*****"}
                            onSaveWifiCredentials={({ssid, password}) => {

                            }} connected={false}/>
                        <GoOnlineCredentialsCard
                            authorized={false}
                            initialUsername={null}
                            hiddenPassword={"H*****"}
                            onSaveCredentials={(u, p) => {
                            }}
                            loading={true}/>

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

                    </div>
                </div>
            </main>
        </div>
    )
}
