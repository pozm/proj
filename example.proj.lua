print("test")

local s = luaScript("pogger")
s.invoke_fn = function() 

    print("pog champ")
    
    print(fs)
    local file = fs:createFile(DIR_PROJECT .. "pogger.lua");
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
    print(pcall(function() 
        return fs:exists(DIR_PROJECT.. "../pogger.lua")
    end))
    print(fs:exists(DIR_PROJECT.. "../pogger1.lua"))

    local remote_content = http:request({
        url="https://httpbin.org/anything",
        method="get",
        headers={pog="oui"},
        body="poggerere"
    })
    print(remote_content.body)

    -- print(remote_content)

    print(permissions.allowed)
    print(permissions.denied)
    print(permissions)

    
end

print(s)

scriptManager:add(s)
local s = luaScript("zip-test")
s.invoke_fn = function() 

    local z = http:request({
        url="https://github.com/curl/curl/releases/download/curl-7_83_1/curl-7.83.1.zip",
        method="get",
        content_type="Text",
        headers={}
    });
    local file = fs:createFile(DIR_PROJECT .. "Curl.zip");
    file:write(z.body.Text);
    print("done writing");
    pcall(function()
        fs:createDir(DIR_PROJECT .. "pog2/")

    end)
    file:unzip(DIR_PROJECT .. "pog2/");

end


scriptManager:add(s)
