import {Label} from "@/components/ui/label.tsx";
import {Input} from "@/components/ui/input.tsx";
import {PasswordInput} from "@/components/custom/PasswordInput.tsx";
import {useState} from "preact/hooks";
import {SettingsCard} from "@/components/custom/SettingsCard.tsx";
import {useEffect} from "preact/compat";
import {generateMaskedPassword, I_GenericResponse} from "@/lib/utils.ts";




interface I_GoLoginStatus {
    authorized: boolean,
    username: string,
    first_letter_of_password: string,
    length_of_password: number,
}

export const GoOnlineCredentialsCard = () => {
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState<null | I_GoLoginStatus>(null);
    const getGoInfo = async (): Promise<I_GoLoginStatus | null> => {
        setLoading(true)
        let response = await fetch("/online-go-status")
        let responseJson = await response.json() as I_GenericResponse<I_GoLoginStatus, any>;
        setLoading(false)
        if (responseJson.is_ok) {
            return responseJson.value
        } else {
            alert(`ERROR UPDATING UPDATING GO STATUS  see console`)
            console.error(responseJson.value)
            return null;
        }

    }
    useEffect(() => {
        getGoInfo().then((info) => {
            setStatus(info)
        })
    }, [])

    const saveGoLogin = async (username: string, password: string) => {
        setLoading(true);

        const response = await fetch("/online-go-login", {
            method: "POST",
            body: JSON.stringify({username, password}),
        });
        if (!response.ok) {
            // TODO: better request error handling (Popup?)
            alert("ERROR UPDATING WIFI CREDS REQ")
            return
        }
        setLoading(false)
    }


    let [username, setUsername] = useState<null | string>();
    useEffect(() => {
        if (username == null) {
            setUsername(status?.username)
        }
    }, [status]);
    let [password, setPassword] = useState("");

    const hiddenPassword = generateMaskedPassword(status?.first_letter_of_password ?? null, status?.length_of_password ?? null);

    return <SettingsCard
        title={"Online Go Account"}
        description={"Used to access your online-go.com information."}
        noErrorBadgeText={"Authorized"}
        errorBadgeText={"Error"}
        onSave={async () => {
            await saveGoLogin(username ?? "", password)
        }}
        error={!(status?.authorized ?? false)} loading={loading}>

        <Label>Username</Label>
        <Input placeholder="Your username" value={username ?? ""}
               onChange={(event) => {
                   setUsername((event.currentTarget as HTMLInputElement).value)
               }}/>
        <Label>Password</Label>
        <PasswordInput placeholder={hiddenPassword ?? "Your password"}
                       setValue={setPassword}/>

        
    </SettingsCard>

}