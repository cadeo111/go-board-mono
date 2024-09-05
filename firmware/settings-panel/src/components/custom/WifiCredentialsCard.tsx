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

interface WifiCredentialsCardParams {
    onSaveWifiCredentials: (creds: { ssid: string; password: string }) => void;
    hiddenPassword: string | null;
    initialSSID: string | null;
    connected: boolean;
    loading:boolean;
}



export const WifiCredentialsCard = ({onSaveWifiCredentials, hiddenPassword, initialSSID, connected, loading}: WifiCredentialsCardParams,
) => {
    let [ssid, setWifiSSID] = useState<null|string>();

    useEffect(() => {
        if(ssid == null){
            setWifiSSID(initialSSID)
        }
    }, [initialSSID]);

    let [password, setWifiPassword] = useState("");


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
                    <AlertDialogAction onClick={() => onSaveWifiCredentials({ssid:ssid??"", password})}>Save & Restart</AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>}>
        <Label>Network Name (SSID)</Label>
        <Input placeholder="Wifi SSID" value={ssid ?? ""}
               onChange={(event) => {
            setWifiSSID((event.currentTarget as HTMLInputElement).value)
        }}/>
        <Label>Password</Label>
        <PasswordInput placeholder={hiddenPassword ?? "Wifi password"} setValue={setWifiPassword}/>
    </SettingsCard>


}