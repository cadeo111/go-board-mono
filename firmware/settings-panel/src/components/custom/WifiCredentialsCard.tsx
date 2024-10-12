import {Label} from "@/components/ui/label.tsx";
import {Input} from "@/components/ui/input.tsx";
import {PasswordInput} from "@/components/custom/PasswordInput.tsx";
import {useState} from "preact/hooks";
import {SettingsCard} from "@/components/custom/SettingsCard.tsx";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger
} from "@/components/ui/alert-dialog.tsx";
import {Button} from "@/components/ui/button.tsx";
import {useEffect} from "preact/compat";
import {generateMaskedPassword, I_GenericResponse} from "@/lib/utils.ts";


interface I_WifiStatus {
    connected: boolean,
    ssid: string,
    first_letter_of_password: string,
    length_of_password: number,
}


export const WifiCredentialsCard = () => {
    const getWifiInfo = async (): Promise<I_WifiStatus | null> => {
        setLoading(true)
        let response = await fetch("wifi-status")
        console.log("HELLO FROM STATUS", response)
        let responseJson = await response.json() as I_GenericResponse<I_WifiStatus, any>;

        setLoading(false)
        if (responseJson.is_ok) {
            return responseJson.value
        } else {
            alert(`ERROR UPDATING WIFI CREDS see console`)
            console.error("ERROR JSON", responseJson.value)
            return null;
        }

    }


    let [loading, setLoading] = useState(false);
    const saveWifiCredentials = async (ssid: string, password: string) => {
        setLoading(true);

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


    // let [ssid, setWifiSSID] = useState<null|string>();

    // useEffect(() => {
    //     if(ssid == null){
    //         setWifiSSID(initialSSID)
    //     }
    // }, [initialSSID]);

    let [password, setWifiPassword] = useState("");
    let [hiddenPassword, setHiddenPassword] = useState("Wifi password");
    let [ssid, setSsid] = useState("")
    let [connected, setConnected] = useState(false)

    useEffect(() => {
        getWifiInfo().then((info) => {
            console.log("GOT WIFI STATUS", info)
            if (info?.ssid != undefined) {
                setSsid(info!.ssid);
            }
            if (info?.connected != undefined) {
                setConnected(info!.connected)
            }
            const possibleHiddenPassword = generateMaskedPassword(info?.first_letter_of_password ?? null, info?.length_of_password ?? null)
            if (possibleHiddenPassword != null) {
                setHiddenPassword(possibleHiddenPassword)
            }
        })
    }, [])


    return <SettingsCard
        title={"Wifi Credentials"}
        description={"Used to access the internet."}
        error={!connected}
        noErrorBadgeText={"Connected"}
        errorBadgeText={"Not Connected"}
        loading={loading}
        buttonElement={<AlertDialog>
            <AlertDialogTrigger asChild>
                <Button>Save</Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                    <AlertDialogDescription>
                        This action will restart the device and
                        attempt to connect with the provided credentials.
                        you will not have access to this panel during the restart.
                    </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction onClick={() => saveWifiCredentials(ssid, password)}>Save & Restart</AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>}>
        <Label>Network Name (SSID)</Label>
        <Input placeholder="Wifi SSID" value={ssid}
               onChange={(event) => {
                   setSsid((event.currentTarget as HTMLInputElement).value)
               }}/>
        <Label>Password</Label>
        <PasswordInput placeholder={hiddenPassword} setValue={setWifiPassword}/>
    </SettingsCard>


}