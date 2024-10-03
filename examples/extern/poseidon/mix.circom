pragma circom 2.0.0;

template Mix(t, M) {
    signal input in[t];
    signal output out[t];

    var lc;
    for (var i=0; i<t; i++) {
        lc = 0;
        for (var j=0; j<t; j++) {
            lc += M[j][i]*in[j];
        }
        out[i] <== lc;
    }
}

template MixLast(t, M, s) {
    signal input in[t];
    signal output out;

    var lc = 0;
    for (var j=0; j<t; j++) {
        lc += M[j][s]*in[j];
    }
    out <== lc;
}

template MixS(t, S, r) {
    signal input in[t];
    signal output out[t];


    var lc = 0;
    for (var i=0; i<t; i++) {
        lc += S[(t*2-1)*r+i]*in[i];
    }
    out[0] <== lc;
    for (var i=1; i<t; i++) {
        out[i] <== in[i] +  in[0] * S[(t*2-1)*r + t + i -1];
    }
}
