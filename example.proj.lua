print("test")

local s = luaScript("pogger")
s.invoke_fn = function() 

    print("pog champ")

    local file = fs:createFile(DIR_PROJECT.. "pogger.lua");
    file:write("print'pogger'")
    file:clear()
    file:write("yays!")
    local content = file:read()
    print(content)
    file:write("pog champ")
    local content = file:read()
    print(content)
    file:seek(3)
    file:write("gers!")
    local content = file:read()
    print(content)

    local files = fs:openDir(DIR_PROJECT)
    print(files)

    print(file)
    print(fs)

    print(fs:exists(DIR_PROJECT.. "pogger.lua"))
    print(fs:exists(DIR_PROJECT.. "pogger1.lua"))

    local remote_content = http:request({
        url="https://raw.githubusercontent.com/pozm/proj/master/readme.md",
        method="get",
        headers={}
    })
    print(remote_content.body)

end

print(s)

scriptManager:add(s)
local s = luaScript("tauri-poggers")
s.invoke_fn = function() print("test") end

print(s)

scriptManager:add(s)