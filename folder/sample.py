import os
for i in range(100):
    fpath = os.path.join(os.getcwd(),f"failed_jobs{i}.csv")
    print(fpath)
    if os.path.exists(fpath): 
        os.remove(fpath)
        continue
    else:
        with open(fpath,'w') as writer:
            writer.write(f"This is my nth {i} out of 10 files \n")