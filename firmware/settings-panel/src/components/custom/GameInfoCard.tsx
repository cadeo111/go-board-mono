import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "@/components/ui/card.tsx";
import {Badge} from "@/components/ui/badge.tsx";
import {Label} from "@/components/ui/label.tsx";
import {Combobox} from "@/components/custom/Combobox.tsx";
import {Button} from "@/components/ui/button.tsx";

export const GameInfoCard = () => {




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
                                    label: <div className={"flex justify-between flex-1"}> cd113
                                        <div className={"space-x-1"}><Badge
                                            className={"border-violet-500 hover:bg-violet-200 border-2 bg-transparent text-primary rounded-md px-1 py-0"}>4/16</Badge>
                                            <Badge
                                                className={"border-orange-500 hover:bg-orange-200 border-2 bg-transparent text-primary rounded-md px-1 py-0"}>8x8</Badge>
                                        </div></div>, value: 0,
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
}