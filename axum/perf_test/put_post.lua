wrk.method = "PUT"
wrk.headers["Content-Type"] = "application/json"
math.randomseed(os.time())

wrk.body = [[
    {
    "title": "Updated Title",
    "content": "Updated Content"
    }
    ]]

request = function()

  local id = math.random(500, 600) 

  return wrk.format(nil, "/posts/" .. id)
end
