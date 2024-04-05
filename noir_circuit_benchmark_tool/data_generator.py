import random
import numpy as np
import math
import json

len = 100
vec1 = np.random.randint(0, 256, size=len)
vec2 = np.random.randint(0, 256, size=len)

for i in range(len):
    vec2[i] = vec1[i] + int(math.log(vec2[i] + 1,2) - 4)
    if(vec2[i] < 0):
        vec2[i] = 0
    if(vec2[i] > 255): 
        vec2[i] = 255
        

print(np.sum((vec2-vec1)**2))

#write to noir
with open("../noir/Prover.toml", "w") as f:
    f.write("x = [")
    for i in range(len):
        if i != len-1:
            f.write(str(vec1[i]) + ", ")
        else:
            f.write(str(vec1[i]))
    f.write("]\n")
    f.write("y = [")
    for i in range(len):
        if i != len-1:
            f.write(str(vec2[i]) + ", ")
        else:
            f.write(str(vec2[i]))
    f.write("]\n")

#write to sp1
vec1 = np.array(vec1)
vec2 = np.array(vec2)
print(vec1,vec2)
combined_vec = np.concatenate((vec1, vec2), axis=0).tolist()
combined_vec = bytes(combined_vec)
with open("../sp1/script/test.dat", "wb") as f:
    f.write(combined_vec)

#write to circom
vec1_array = vec1.astype(str)
vec2_array = vec2.astype(str)
data_dict = {
    "in1": vec1_array.tolist(),
    "in2": vec2_array.tolist()
}
with open("../circom/input.json", "w") as f:
    json.dump(data_dict, f)