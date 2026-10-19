[Code]
{ The PATH of the current user, so no admin rights are needed }
const EnvironmentKey = 'Environment';

procedure EnvAddPath(Path: string);
var
    Paths: string;
begin
    { Retrieve current path (use empty string if entry not exists) }
    if not RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths)
    then Paths := '';

    { Skip if string already found in path }
    if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

    { Append the path, without creating empty entries }
    if Paths = '' then Paths := Path
    else if Copy(Paths, Length(Paths), 1) = ';' then Paths := Paths + Path
    else Paths := Paths + ';' + Path;

    { Write it as REG_EXPAND_SZ, so entries like %USERPROFILE%\bin keep working }
    if RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] added to PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while adding the [%s] to PATH: [%s]', [Path, Paths]));
end;

procedure EnvRemovePath(Path: string);
var
    Paths, Rest, Entry, NewPaths: string;
    P: Integer;
begin
    { Skip if registry entry not exists }
    if not RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then
        exit;

    { Skip if string not found in path }
    if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') = 0 then exit;

    { Rebuild the path without the entry }
    NewPaths := '';
    Rest := Paths;
    while Rest <> '' do
    begin
        P := Pos(';', Rest);
        if P = 0 then
        begin
            Entry := Rest;
            Rest := '';
        end
        else
        begin
            Entry := Copy(Rest, 1, P - 1);
            Rest := Copy(Rest, P + 1, Length(Rest));
        end;

        if (Entry <> '') and (CompareText(Entry, Path) <> 0) then
        begin
            if NewPaths <> '' then NewPaths := NewPaths + ';';
            NewPaths := NewPaths + Entry;
        end;
    end;

    { Overwrite path environment variable }
    if RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', NewPaths)
    then Log(Format('The [%s] removed from PATH: [%s]', [Path, NewPaths]))
    else Log(Format('Error while removing the [%s] from PATH: [%s]', [Path, NewPaths]));
end;
