import {Label} from "@/components/ui/label.tsx";
import {Input} from "@/components/ui/input.tsx";
import {PasswordInput} from "@/components/custom/PasswordInput.tsx";
import {useState} from "preact/hooks";
import {SettingsCard} from "@/components/custom/SettingsCard.tsx";
import {useEffect} from "preact/compat";

interface GoOnlineCredentialsCardParams {
    onSaveCredentials: (username: string, password: string) => void;
    authorized: boolean;
    initialUsername: string | null;
    hiddenPassword: string | null,
    loading: boolean;
}

export const GoOnlineCredentialsCard = ({
                                            onSaveCredentials,
                                            authorized,
                                            initialUsername,
                                            loading,
                                            hiddenPassword
                                        }: GoOnlineCredentialsCardParams) => {

    let [username, setUsername] = useState<null | string>();
    useEffect(() => {
        if (username == null) {
            setUsername(initialUsername)
        }
    }, [initialUsername]);


    let [password, setPassword] = useState("");

    return <SettingsCard
        title={"Online Go Account"}
        description={"Used to access your online-go.com information."}
        noErrorBadgeText={"Authorized"}
        errorBadgeText={"Error"}
        onSave={() => {
            onSaveCredentials(username ?? "", password)
        }}
        error={!authorized} loading={loading}>

        <Label>Username</Label>
        <Input placeholder="Your username" value={username ??""}
               onChange={(event) => {
                   setUsername((event.currentTarget as HTMLInputElement).value)
               }}/>
        <Label>Password</Label>
        <PasswordInput placeholder={hiddenPassword ?? "Your password"}
                       setValue={setPassword}/>

    </SettingsCard>

}