-- Simple performance test
local function factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end

local start_time = os.clock()
local result = factorial(20)
local end_time = os.clock()

print("Factorial result:", result)
print("Execution time:", end_time - start_time)