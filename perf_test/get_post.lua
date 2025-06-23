-- get_posts.lua
math.randomseed(os.time())

request = function()
  local id = math.random(500, 1000)  -- ID 500 ~ 10000 사이에서 랜덤 선택
  local path = "/posts/" .. id
  return wrk.format("GET", path)
end
--wrk -t4 -c500 -d60s -s get_post.lua http://localhost:3000