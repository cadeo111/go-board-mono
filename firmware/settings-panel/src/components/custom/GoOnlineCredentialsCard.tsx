import {Label} from "@/components/ui/label.tsx";
import {Input} from "@/components/ui/input.tsx";
import {PasswordInput} from "@/components/custom/PasswordInput.tsx";
import {useState} from "preact/hooks";
import {SettingsCard} from "@/components/custom/SettingsCard.tsx";

interface GoOnlineCredentialsCardParams {
    onSaveCredentials: (username: string, password: string) => void;
    authorized: boolean;
    initialUsername: string | null;
    loading: boolean;
}

export const GoOnlineCredentialsCard = ({onSaveCredentials, authorized, initialUsername, loading}: GoOnlineCredentialsCardParams) => {
    let [username, setUsername] = useState(initialUsername ?? "");
    let [password, setPassword] = useState("");

    return <SettingsCard
        title={"Online Go Account"}
        description={"Used to access your online-go.com information."}
        noErrorBadgeText={"Authorized"}
        errorBadgeText={"Error"}
        onSave={() => {
            onSaveCredentials(username, password)
        }}
        error={!authorized} loading={loading}>

        <Label>Username</Label>
        <Input placeholder="Your username" value={initialUsername}
               onChange={(event) => {
                   setUsername((event.currentTarget as HTMLInputElement).value)
               }}/>
        <Label>Password</Label>
        <PasswordInput placeholder="Your password"
                       setValue={setPassword}/>

    </SettingsCard>

}