# Firmware for 16x16 LED array + rotary encoder as a Go Board connected to online-go.com

how should work 
```mermaid
flowchart TD
    PON(Power On) --> REB{Rotatary Encoder\n Button Held};
    REB ----->|True| SP;
    REB ----->|False| A
    PON --->|Restart To Settings| SP;
subgraph Settings_Panel;
SP(Settings Panel Start) --> 
        SWI(Start wifi AP + Try to Connect Wifi) -->
CP(Set up Captive Portal Redirect DNS) -->
SWS(Start web server);
end


subgraph Main_Program;

A(Main Program Start) --> R{Connect to wifi}
R -->|Connected|CG{Get current Game}
RESTART(Restart in settings mode !TODO!)
R -->|Not Connected|RESTART
CG --->|Got Game|GG
CG --->|Unauthorized / Connection Failure|RESTART
end


GG(Start Normal Game Loop)


```




```mermaid
flowchart TD
;
    GG(Start Normal Game Loop) --> GS
    GS(Is Game completed)
    GS --->|No| SLS
    GS ---------->|Yes| SCG
    subgraph Active Game
        SLS(Show Current\n Game State)
        SLS --->|move RE| SC(Show/Move current \nselection cursor) --> SLS
        SLS --->|Press RE Btn for 5 secs| SSCS("Show Current Score")
        SSCS --->|Wait 5 sec| SCPL("Show Last Move\n and Player turn")
        SCPL --->|Press RE Btn for 5 secs| SLS
        SSCS --->|Press RE Btn for 5 secs| SLS
        SLS --->|Press RE Btn| SSLS("Show *Selected* State")
        SSLS --->|move RE| SPS("Show move\n is a pass now")
        SPS --->|Press RE Btn Again| SP(Save Pass) --> SLS
        SPS --->|move RE| SSLS
        SSLS --->|Press RE Btn Again| VM{"Is Valid Move"}
        VM -->|Yes| SM(Save move) --> SLS
        VM -->|No| IM(show Move\n is Invalid) --> SLS
    end

    subgraph Completed Game
        SCG(Show Completed Game)
        SCG --->|move RE| SGSC("Show Game Score")
        SGSC --->|move RE| SCG
    end



```
