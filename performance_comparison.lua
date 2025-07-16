local function fibonacci(n)
    if n <= 1 then
        return n
    else
        return fibonacci(n-1) + fibonacci(n-2)
    end
end

local start_time = os.clock()
local result = fibonacci(25)
local end_time = os.clock()

print("Fibonacci(25) =", result)
print("Time taken:", end_time - start_time, "seconds")