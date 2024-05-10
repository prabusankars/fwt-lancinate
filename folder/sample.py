import os

for i in range(10):
    fpath = os.path.join(os.getcwd(),f"sample{i}.txt")
    print(fpath)
    if os.path.exists(fpath): 
        os.remove(fpath)
        continue
    else:
        with open(fpath,'w') as writer:
            writer.write(f"This is my nth {i} out of 10 files \n")