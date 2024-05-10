import os

for i in range(100):
    fpath = os.path.join(os.getcwd(),f"sample{i}.txt")
    print(fpath)
    with open(fpath,'w') as writer:
        writer.write("This is my nth {i} out of 100 files \n")