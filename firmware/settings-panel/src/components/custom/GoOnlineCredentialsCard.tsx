import {Label} from "@/components/ui/label.tsx";
import {Input} from "@/components/ui/input.tsx";
import {PasswordInput} from "@/components/custom/PasswordInput.tsx";
import {useState} from "preact/hooks";
import {SettingsCard} from "@/components/custom/SettingsCard.tsx";
import {useEffect} from "preact/compat";
import {generateMaskedPassword, I_GenericResponse} from "@/lib/utils.ts";

interface I_NoSavedAccount {
    noSavedAccount: true
}

interface I_GoLoginStatus {
    authorized: boolean,
    username: string,
    first_letter_of_password: string,
    length_of_password: number,
}

interface I_GoLoginStateUnauthorized extends I_GoLoginStatus {
    authorized: false,
    "response": {
        "error": String, // will be empty string if all went well
        "error_description": String,
    }
    "status_code": any,
}


export const GoOnlineCredentialsCard = () => {
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState<Awaited<ReturnType<typeof getGoInfo>> | null>(null);

    const getGoInfo = async () => {
        setLoading(true);
        let response = await fetch("online-go-status")
        setLoading(false);
        return await parse_answer_response(response)
    }

    const parse_answer_response = async (response: Response): Promise<I_GoLoginStatus | I_GoLoginStateUnauthorized | I_NoSavedAccount | null> => {
        let responseJson = await response.json() as I_GenericResponse<I_GoLoginStatus, any>;
        if (responseJson.is_ok) {
            return responseJson.value
        } else {
            if (
                response.status == 511 // NETWORK_AUTHENTICATION_REQUIRED /  there are no existing credentials
            ) {
                return {noSavedAccount: true}
            }
            if (
                response.status == 401 // UNAUTHORIZED /  there are credentials but they are invalid
            ) {
                return responseJson.value as I_GoLoginStateUnauthorized
            }
            alert(`ERROR GETTING GO STATUS see console`)
            console.error("ERROR JSON", responseJson.value)
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

        setLoading(false)

        let info = await parse_answer_response(response)
        setStatus(info)
    }


    let [username, setUsername] = useState<null | string>();
    useEffect(() => {
        if (username == null) {
            if (status != null) {
                if (!("noSavedAccount" in status && status?.noSavedAccount != undefined)) {

                    setUsername((status as Exclude<typeof status, I_NoSavedAccount>)?.username)
                }
            }
        }
    }, [status]);
    let [password, setPassword] = useState("");

    const getStatusOrNull = (currentStatus: typeof status) => {
        if (currentStatus == null) return null
        if ("noSavedAccount" in currentStatus) return null
        currentStatus = (currentStatus as Exclude<typeof status, I_NoSavedAccount | null>);
        return generateMaskedPassword(currentStatus?.first_letter_of_password ?? null, currentStatus?.length_of_password ?? null)
    }


    //TODO fix error updating account info

    const getErrorAndErrorText = (currentStatus: typeof status): { isError: boolean, errorText: string } => {
        let l: [boolean, string];
        if (currentStatus == null) {
            l = [true, "Unknown Error"]
        } else if ("noSavedAccount" in currentStatus && currentStatus?.noSavedAccount != undefined) {
            l = [true, "No Saved Account"]
        } else if (!(currentStatus as Exclude<typeof status, I_NoSavedAccount | null>).authorized) {
            l = [true, "Incorrect Login Information"]
        } else {
            l = [false, "Authorized"]
        }
        return {isError: l[0], errorText: l[1]}
    }

    return <SettingsCard
        title={"Online Go Account"}
        description={"Used to access your online-go.com information."}
        noErrorBadgeText={getErrorAndErrorText(status).errorText}
        errorBadgeText={getErrorAndErrorText(status).errorText}
        onSave={async () => {
            await saveGoLogin(username ?? "", password)
        }}
        error={getErrorAndErrorText(status).isError} loading={loading}>

        <Label>Username</Label>
        <Input placeholder="Your username" value={username ?? ""}
               onChange={(event) => {
                   setUsername((event.currentTarget as HTMLInputElement).value)
               }}/>
        <Label>Password</Label>
        <PasswordInput placeholder={getStatusOrNull(status) ?? "Your password"}
                       setValue={setPassword}/>


    </SettingsCard>

}