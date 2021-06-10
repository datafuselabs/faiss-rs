#!/usr/bin/env sh
repo_url=https://github.com/Enet4/faiss.git
repo_rev=c_api_head

git clone $repo_url faiss --branch $repo_rev --depth 1

cd faiss

# Build
cmake . -DFAISS_ENABLE_C_API=ON -DBUILD_SHARED_LIBS=ON \
    -DFAISS_ENABLE_PYTHON=OFF -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release

make faiss_c
mkdir -p "$HOME/.faiss_c"
cp faiss/libfaiss.so c_api/libfaiss_c.so "$HOME/.faiss_c/"

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$HOME/.faiss_c

echo "libfaiss_c.so installed in $HOME/.faiss_c/"

cd ..

# clean up
rm -rf faiss
