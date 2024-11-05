local x

if math.round(math.random(0, 1)) == 0 then
    x = 1
else
    x = -1
end

function start(self)
    print("Rotating script started")
    print(self)
end

function update(self, delta_time)
    self:rotate(vector(0, x * delta_time, 0))
end