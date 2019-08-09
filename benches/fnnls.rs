// Copy of the fnnls.rs in the main directory.
// Used because I couldn't figure out pathing for the benchmarks.

#![allow(non_snake_case)]

use ndarray::{array, Dimension, prelude::*};
use ndarray_linalg::Solve;
use std::f64::{consts, EPSILON};


pub fn fnnls(xtx: &Array2<f64>, xty: &Array1<f64>)
    -> (Array1<f64>, Array1<f64>) {
    let (M, N) = (xtx.rows(), xtx.cols());
    let mut P = Array::zeros(M);               // passive; indices with vals > 0
    let mut Z = Array::from_iter(0..N) + 1;            // active; i w/ vals <= 0
    let mut ZZ = Array::from_iter(0..N) + 1;               // working active set
    let mut x: Array1<f64> = Array::zeros(N);         // initial solution vector
    let mut w = xty - &(xtx.dot(&x));                           // weight vector
    let mut it = 0;                               // iterator for the while loop
    let itmax = 30 * N;                                    // maximum iterations

    // Continue if indices in the active set or values > than machine epsilon
    while Z.iter().any(|&i| i > 0) && ZZ.iter().any(|&i| &w[i - 1] > &EPSILON)
    {
        let t = max_index(&(ZZ.mapv(|i| w[i - 1]))) + 1;
        P[ZZ[t - 1] - 1] = ZZ[t - 1];                 // move to the passive set
        Z[ZZ[t - 1] - 1] = 0;                      // remove from the active set
        ZZ = Array::from_vec(find_nonzero(&Z)) + 1;
        let mut PP = Array::from_vec(find_nonzero(&P)) + 1;
        let mut PPcopy = Array::from_vec(find_nonzero(&P)) + 1;
        let mut s: Array1<f64> = Array::zeros(N);              // trial solution
        match PP.len() {
            0 => s[0] = 0.0,
            1 => s[PP[0] - 1] = xty[PP[0] - 1] / xtx[[PP[0] - 1, PP[0] - 1]],
            _ => {
                let xtx_pp_solution = slice_with_array(xtx, &(PPcopy - 1))
                    .solve_into(PP.mapv(|i| xty[i - 1]))
                    .unwrap();              // solve PP-reduced set of xtx @ xty
                for (i, &value) in PP.indexed_iter() {
                    s[value - 1] = xtx_pp_solution[i];
                }
            }
        }
        for &i in ZZ.iter() {
            s[i - 1] = 0.0;                      // set active coefficients to 0
        }

        while (&PP).iter().any(|&i| &s[i - 1] <= &EPSILON) && it < itmax {
            it += 1;
            let s_mask = s.mapv(|e| e <= EPSILON);
            let tmp = P
                .indexed_iter()
                .map(|(i, &v)| if s_mask[[i]] { v } else { 0 })
                .collect::<Vec<usize>>();
            let QQ = Array::from_vec(find_nonzero_vec(&tmp)) + 1;
            let xQQ = QQ.mapv(|i| x[i - 1]);
            let alpha = min(&(&xQQ / &(&xQQ - &QQ.mapv(|i| s[i - 1]))));
            x = &x + &(alpha * (&s - &x));
            let mask = P.mapv(|i| i != 0) & x.mapv(|i| i.abs() < EPSILON);
            for (i, &v) in mask.indexed_iter() {
                if v {
                    Z[i] = i + 1;
                    P[i] = 0;
                }
            }
            PP = Array::from_vec(find_nonzero(&P)) + 1;
            PPcopy = Array::from_vec(find_nonzero(&P)) + 1;
            ZZ = Array::from_vec(find_nonzero(&Z)) + 1;
            match PP.len() {
                // verbatim repeat of the previous match statement
                0 => s[0] = 0.0,
                1 => {
                    s[PP[0] - 1] = xty[PP[0] - 1] / xtx[[PP[0] - 1, PP[0] - 1]]
                }
                _ => {
                    let xtx_pp_solution = slice_with_array(xtx, &(PPcopy - 1))
                        .solve_into(PP.mapv(|i| xty[i - 1]))
                        .unwrap();
                    for (i, &value) in PP.indexed_iter() {
                        s[value - 1] = xtx_pp_solution[i];
                    }
                }
            }
            for &i in ZZ.iter() {
                s[i - 1] = 0.0;
            }
        }
        x = s;                               // assign current solution (s) to x
        w = xty - &(xtx.dot(&x));                           // recompute weights
    }
    (x, w)
}


pub fn build_outer_array(a: &Array1<f64>, b: &Array1<f64>) -> Array2<f64> {
    let mut outerArray = Array::zeros((a.len(), b.len()));
    for i in 0..a.len() {
        for j in 0..b.len() {
            outerArray[[i, j]] = a[i] * b[j];
        }
    }
    outerArray
}


pub fn find_nonzero_vec(vec: &Vec<usize>) -> Vec<usize> {
    vec.iter()
       .enumerate()
       .filter(|(_, &value)| value != 0)
       .map(|(i, _)| i)
       .collect::<Vec<usize>>()
}


pub fn find_nonzero(array: &Array1<usize>) -> Vec<usize> {
    array.indexed_iter()
         .filter(|(_, &value)| value !=0)
         .map(|(i, _)| i)
         .collect::<Vec<usize>>()
}


pub fn slice_with_array(a: &Array2<f64>, b: &Array1<usize>) ->
                        Array2<f64> {
    let mut newArray = Array::zeros((b.len(), b.len()));
    for i in 0..b.len() {
        for j in 0..b.len() {
            newArray[[i, j]] = a[[b[j], b[i]]]
        }
    }
    newArray
}


pub fn max_index(array: &Array1<f64>) -> usize {
    let mut index = 0;
    for (i, &value) in array.iter().enumerate() {
        if value > array[index] {
            index = i;
        }
    }
    index
}


pub fn min<D: Dimension>(array: &Array<f64, D>) -> f64 {
    array.iter().cloned().fold(1.0/0.0, f64::min)
}


pub fn test_setup() -> (Array2<f64>, Array1<f64>) {
    let nu = Array::linspace(0.005, 0.15, 600);
    let t = Array::linspace(0.0, 80.0, 1000);
    let R = (-build_outer_array(&t, &nu)).mapv(|a| consts::E.powf(a));
    let rate_id = vec![50, 400];
    let mut weights: Array1<f64> = Array::zeros(nu.raw_dim());
    let true_rates: Vec<f64> = rate_id.iter()
                                      .map(|&x| nu[[x]])
                                      .collect();
    println!("True rates at: {}, {}", true_rates[0], true_rates[1]);
    weights[rate_id[0]] = 0.6;
    weights[rate_id[1]] = 0.4;
    let noise_g = python_random_array();
    let Yobs = R.dot(&weights) + noise_g;
    let RtR = R.t().dot(&R);
    let Rtb = R.t().dot(&Yobs);
    // the sums of the two fnnls() outputs should be 1.000034 and -0.102106 
    // fnnls(&RtR, &Rtb); // function call if not benchmarking
    (RtR, Rtb)                                        // output for benchmarking
}


fn python_random_array() -> Array1<f64> {
    array![
      -9.3641758783710662e-04, -1.6456497080460733e-03, -1.3076020431153755e-04,
      -1.4267238483956157e-03,  1.8127017746563318e-03,  1.5324734618583436e-03,
       1.6521081429751907e-03, -2.6473655709615836e-03, -3.5048890479229453e-03,
       2.0048981367986980e-03,  1.0896189098128113e-03,  3.7903218067930626e-03,
      -1.5387149092665644e-03, -2.8061918358721358e-03, -1.2649350104806108e-03,
      -1.1177473355716823e-03, -2.4664627695728672e-03, -8.7900704226996217e-04,
       1.8295745196397525e-03,  5.3008186053788494e-04, -2.7667403029330551e-03,
       1.3710236079906830e-03,  9.1218182909807055e-04, -9.2274852690927384e-04,
       1.8940060969261120e-04, -3.0856232118783708e-03,  4.9587391964916894e-03,
       9.1373351969253132e-04, -6.2774555651546563e-04,  4.2074760529244274e-05,
       1.9215863225710732e-03,  1.1696579397137198e-04, -8.9206433814186133e-04,
       6.3839427904538969e-04,  1.6823372452407978e-03, -3.0655239882113449e-03,
      -5.6316851681179416e-04,  3.4889054195583634e-03, -1.3484778188728535e-03,
       1.1768024591793163e-03,  3.6087269272808533e-03,  4.1125005211693861e-03,
       2.9091633657510993e-03, -2.7682338452663020e-04,  6.8574376544271686e-04,
      -1.4552369636181065e-03, -2.8078921997138073e-03, -2.4812223836362603e-03,
      -8.8696433372429707e-04, -9.4650324065479580e-05,  1.5153687203314763e-03,
      -3.0417185346100705e-04, -5.4255801323450687e-04, -1.1996799471072131e-03,
      -4.0538082501243352e-03,  6.6068480925469053e-04, -6.6166200300677650e-04,
      -6.9884326196223811e-05,  5.7949607703734838e-04, -1.2125398970098660e-03,
      -5.3680855447442108e-04,  2.3829550445612865e-03,  3.1522662213766083e-04,
       2.3496495088724729e-03,  2.6418200434356752e-03, -1.6960745748067646e-03,
       1.4914147938031531e-03, -6.2324260312830834e-04, -2.1021307682644666e-03,
      -2.1529936511835652e-03,  9.0121930616687850e-04,  8.1748163583442363e-04,
      -2.8590830359281061e-03,  2.0348544220752259e-03, -1.6354147106807654e-04,
      -7.6783686745302275e-04, -4.7645034855065747e-04,  1.7577235049115433e-05,
       1.0409367653160654e-03,  8.0740904394541765e-04, -8.0356435000144253e-04,
      -1.3944905293332842e-03,  1.2919654549712963e-03, -5.5106628022406749e-04,
      -7.5842058094697292e-04,  3.9360700314213376e-03,  4.0499388469014057e-04,
       7.4539454615755898e-04,  1.9626919544504674e-03,  1.4729246426585275e-03,
       2.8226016060833030e-03, -2.5303544904889099e-04,  1.1063146836765392e-03,
      -1.6310719725758254e-03,  1.0774050368388217e-03, -4.4355899314013287e-03,
      -1.8125843681348508e-03, -2.9242497639198122e-03, -1.3551297570796705e-03,
       3.0456076056777667e-03, -4.5213103347873838e-04, -1.7572606290667129e-03,
      -1.9432095668146909e-03,  1.0832970372125324e-04,  1.2218148521241320e-04,
       2.3476307619717477e-03,  2.2871824890197054e-03,  7.7176938010241108e-04,
       1.0680941714069643e-03,  1.0871873807887583e-03, -5.3371060999560162e-04,
      -8.0680027948174687e-04, -4.5080901124717901e-04, -1.2299966755716788e-03,
       2.8475653996243968e-03,  2.2753526224298139e-03, -1.3100491704668257e-03,
       1.9243464133344489e-03,  1.2934329923193720e-03,  9.9501427802648813e-04,
      -2.4240119595853441e-03, -3.6359854163201116e-03,  1.8135086108187310e-03,
       2.0580559605356420e-03,  9.9425965701532533e-04,  3.2113066709357992e-03,
      -1.8908236342173658e-04,  3.9314227555489262e-04,  2.1728882621947429e-03,
       1.0683539336091544e-03,  7.3501278802886292e-04, -5.0929892060905908e-03,
      -2.1466464099679348e-04, -2.2535490362485060e-04, -1.9098255408637744e-05,
      -1.0828391694097802e-03, -1.8882784919678930e-03,  1.0723772635386083e-03,
      -8.3130640360992139e-04,  6.4693870959824172e-04,  8.9893603484835513e-04,
       4.9544655929140901e-03, -3.6101875118888725e-04,  1.6527174015605493e-05,
      -8.7614884834075961e-04,  2.1820372715583894e-03,  2.4235718714954660e-04,
      -1.0697860118480818e-03,  2.9259550754155237e-03, -2.0227571472232978e-03,
       2.4999132397700931e-04, -3.3436279152719225e-03,  3.4464652348247097e-03,
       1.4018729878133981e-03,  3.7592464270330690e-03,  3.3607515139308574e-03,
      -8.2693118522137495e-04, -1.0279585071029189e-03, -8.9664973175891064e-04,
      -8.6900055860523518e-04,  9.9453483091594503e-04,  8.9926126415283420e-04,
      -8.4936744052093414e-04, -3.4554469949458233e-03, -1.2075336756492156e-03,
       9.6104714329276180e-04,  3.9180718747147360e-04,  3.0701640745611079e-03,
      -1.9031225927419740e-04, -2.4376084710549316e-03, -1.9465215373519230e-03,
       2.6752178340431337e-03, -4.6035779809563076e-05, -1.5908449547730216e-03,
       1.2308652830265309e-03, -1.2829386189796182e-03, -9.5964343971701718e-04,
      -1.5882813476290588e-03, -1.4158454773014824e-03, -3.2340068649520080e-04,
       6.2286127176563782e-04, -4.7171879118045124e-03,  1.8136496182308733e-03,
       2.6566008322076590e-03,  1.0622737330682971e-03,  1.2658016327067132e-03,
       1.3842663597308882e-03, -2.3426635935468493e-03,  1.7233446195029996e-03,
      -4.2135496290934939e-03, -9.4147986117269853e-04, -1.0055585026904714e-03,
      -5.5459686107200523e-04,  2.3141685262835039e-03,  2.2622040429984944e-03,
      -2.2564043550511656e-03, -5.3672123986181638e-04,  1.6285891167210503e-03,
      -1.9403137654868743e-03,  7.8078718440129217e-06,  1.7001089474339129e-03,
       2.7781808169496680e-03,  4.2388999582397351e-04,  2.7182035007354967e-03,
       3.3260360675872717e-05, -8.3936899269001206e-04, -2.9113998025889082e-04,
      -1.7994131713368249e-03, -3.8441113005099691e-04,  2.2965671378780868e-03,
       2.4516511283193196e-03, -2.4410942305929371e-03,  1.5824538191785825e-03,
      -5.0289266344383363e-04, -1.9513257474605790e-03, -1.5739488470425519e-04,
      -9.5181236908752087e-04, -3.7804356598779404e-03, -1.2257980009995766e-04,
      -7.1982311015706776e-05, -5.6683383665509578e-04, -3.7306325300726100e-04,
       1.4179919167936467e-04, -3.3369180949368958e-04, -3.1187949865654416e-04,
      -5.0793077750377726e-04,  6.4282360498173338e-04, -1.1072638635683913e-03,
      -1.4728165895610613e-03, -2.4648815913352862e-04,  1.4599102198474977e-03,
      -2.4352768650731072e-04, -8.7857566252679486e-04, -2.5513629754800719e-03,
      -1.5810286839633900e-03, -2.6920592952221524e-03,  1.0137707855018352e-03,
      -1.3041510824264363e-03,  2.8604610219716315e-03, -8.9180925029982777e-04,
      -2.3508137615068443e-03,  8.1448987770465239e-04, -2.9986382600724402e-03,
      -1.1442160561306518e-03,  3.5102307492973525e-03,  3.6387940295411015e-05,
       3.6425658914880452e-03,  1.8240302443359705e-04, -1.4942815464234788e-03,
       8.3475066840513125e-04, -3.1056529152048094e-04,  3.5334906887842855e-04,
       1.8278477384298481e-03, -4.4385613294026550e-03, -5.3122984362555126e-03,
      -8.8401579860312743e-05,  3.6296141888580061e-05, -1.6815956524943508e-03,
       2.4103215488137223e-03,  1.5591948919967647e-03,  3.1901770294867973e-03,
       5.8754007045680858e-04, -2.2961270362717241e-03, -6.8568209924812823e-04,
       5.3649126906617488e-04,  3.7851875029359465e-03, -3.8626481582201759e-03,
      -1.4145610112017610e-03,  8.8055644849042135e-04, -1.7635939971674761e-03,
       1.1676079667718995e-03,  6.1694688645709398e-04, -1.2316671441254894e-03,
      -1.0227487039627941e-03, -8.3236167417039670e-04,  2.4487124749744595e-03,
       8.6869342163033809e-04,  6.4370903292550413e-04,  1.6237134569653438e-03,
       1.5062875013804387e-03,  1.1995963097724642e-03,  1.4346882355351614e-03,
      -1.0813305276647295e-03,  2.5999805528672312e-03, -2.1098833752080221e-03,
       1.7247620277067096e-04,  7.7056329806209783e-04, -3.4305852038533226e-03,
       1.0434119561939600e-05, -3.2245833551375878e-03,  8.4460356550429615e-04,
      -2.4900534472688039e-03, -2.6703397773486987e-03, -2.8588901539944953e-03,
       2.4841176497160029e-03,  1.1196335438359846e-03,  3.8981357461192767e-03,
      -2.8619792732550059e-04,  3.1619022927586413e-04,  3.7250696859529733e-04,
       2.8438020643647833e-03, -3.8202036245378193e-03,  6.3563188118163352e-04,
       2.6404053861352572e-03, -2.3240677988623018e-03,  8.6055638681076164e-06,
       6.4114160848110656e-04, -5.1342741604925102e-03, -1.0932835057055018e-03,
      -2.1331170417600983e-03,  7.4390367797860053e-04, -1.4207351847330104e-03,
       1.7926594344192225e-04,  6.6051936788067572e-04,  6.6095596153545918e-04,
      -3.8800041863099139e-03,  4.3578069472884176e-04,  1.8172690439371558e-03,
       5.4879444597716277e-04,  1.8456320854700955e-03, -5.5438095392663387e-05,
      -4.9573528712008050e-04,  1.5282322964784319e-03,  1.4996728461487492e-04,
      -1.0874084530246081e-03,  2.9171079604128364e-03, -1.2370696521682028e-03,
       1.0019418200200258e-04,  1.1365368360647266e-03,  3.6874415894938444e-03,
       5.4846740036053416e-05,  1.3262027688016251e-03,  1.1072459473748127e-03,
       3.7663076314214780e-03,  5.4395861455650815e-05, -1.3791022621465719e-03,
      -1.1120031244218488e-03, -3.2940658526402591e-04, -1.2997423944981215e-03,
      -1.2460483717086064e-03, -2.7053333777954094e-03,  3.0270384201299652e-04,
       4.2685665591747756e-03, -1.8023008132841362e-04,  3.4338771267802088e-03,
       2.2165941628109059e-03,  2.6680062191698604e-04, -3.0619984432495939e-03,
       3.6934796061303070e-04,  1.2748485278763478e-03,  1.4159018404457562e-03,
      -1.9355590980836849e-03, -2.2291535939660324e-03, -2.0286864616177592e-03,
      -3.4708241465341017e-03,  4.3882110479501608e-04, -1.0500826828004489e-03,
       4.1709746506176032e-03,  4.3063426865188486e-04, -7.7023928344865597e-04,
       2.5717877220052397e-03, -3.3619091113370483e-03,  1.1068259774965007e-04,
       1.0447383704261907e-03, -9.6006096715754025e-04,  8.7197950472271669e-04,
       2.0864600643181095e-03, -1.2937278326046414e-03,  1.2486947710418500e-03,
      -1.1848451891092696e-03,  4.0013217083032099e-03,  5.4103139917487380e-04,
       2.1899356557049886e-03, -1.1585575204574169e-03,  2.0155900250484055e-03,
      -5.1006168517525232e-03, -5.8863458610671946e-04, -4.0583832935038198e-03,
       3.0643703635487902e-04, -6.0098150456750410e-04, -2.0399529304807392e-03,
      -2.3042996762645909e-04, -2.7846597668403458e-04, -2.1886810728726359e-03,
       1.3602576258126228e-03, -1.2209977063228834e-04, -9.3828112058203851e-04,
       2.0160229151597983e-03, -2.0814409168044056e-04, -1.8798256275516655e-03,
       1.2080877770130195e-03, -9.4576634412633410e-04, -1.6391977133373540e-03,
       2.1361678826155232e-03,  2.9110441526596850e-03,  5.6878738741446017e-04,
       3.6048698505666095e-04, -1.3490836834095059e-03, -2.0660169313707713e-03,
      -3.2753178947531335e-04,  4.4936608770058534e-03,  1.0926079744772103e-03,
      -1.9579286906035217e-03, -1.8414367684625851e-03,  3.7336285725412578e-05,
       4.2516934533361520e-04,  2.1828460745901388e-03,  1.9680400174623344e-03,
       2.3653372830134017e-03, -1.6372003240579540e-03, -3.3025085098010206e-04,
       4.0097276493864589e-03, -5.5789317666793750e-04, -5.9362095513948226e-04,
       1.3278049540012165e-03,  1.7165164472523577e-03,  3.4363905258895321e-03,
      -5.4183722253984840e-04,  1.7709682486458691e-04, -5.5456453589953179e-04,
      -1.5643580947138052e-03, -3.7143754238543407e-03, -4.2970294265314924e-03,
      -1.7663521090823550e-03,  1.3501990598122085e-04,  4.1178426565946036e-04,
       4.6312789790316179e-03, -1.8896917175467815e-04, -1.0936371129208768e-04,
       1.8429005282516685e-03, -6.8077417014616960e-04, -3.6797207820213900e-03,
      -5.4275172611030931e-04,  1.2254576152568586e-03,  2.3459347912673702e-03,
      -2.5601385945956167e-03,  1.2483521625776535e-03, -1.9026284694594468e-03,
      -2.7829598488628283e-03,  8.6873353784621852e-04,  1.1007208798533169e-03,
      -3.5052314621494144e-03,  4.6089681111758973e-04, -7.2199142249681461e-04,
      -1.6117362368343039e-04, -2.0583696454451236e-03,  6.7075730780475728e-04,
       2.8384219360065785e-03, -1.6258107101374656e-03,  2.8332193416284969e-03,
       4.1661648412092583e-04, -2.1760187133363184e-03,  1.4373676756227424e-03,
       2.5672574862703867e-03, -1.5055924378820115e-03,  1.8180395924679816e-03,
       2.0926006977185218e-04,  3.3661087539535823e-03, -2.3954380537405061e-03,
       3.7480117596248148e-04, -6.2174893078124286e-04, -1.0170308611182519e-03,
       1.4576470163670915e-04,  3.1600047040013994e-03, -1.0364296205149926e-03,
       2.9703520762680330e-03,  6.2966564888837474e-04, -2.1152686306863686e-03,
      -8.1733276513596288e-04, -5.6554187767320779e-04, -3.8240004654880059e-03,
       8.4348922569066998e-05,  1.9083785640111170e-03, -2.1903049494988122e-03,
      -2.9658584204647837e-04,  4.8491739018513807e-04,  4.5463092347585739e-04,
       5.1822453834245398e-05,  1.3502860368459066e-03, -4.6890629849342404e-04,
      -2.6979269078676155e-03, -1.7833256245954323e-03, -9.6513772770678407e-04,
       2.7403947629818504e-04,  3.2185170429529977e-03,  2.9593909238388380e-03,
      -4.7166163900631396e-03,  1.1575172168410634e-03, -1.4057900072220852e-03,
       8.8255407001978361e-04, -2.1301201654564477e-03, -1.5289178198726605e-03,
       2.6419857636723499e-05,  3.2817364929702427e-03,  8.0806920764509894e-04,
      -1.5205736341977563e-03, -2.6215239817415077e-03, -2.6154769169440427e-03,
       1.0869456543618316e-03, -1.1812327306358094e-03,  2.3856521199408015e-03,
       3.2113043585699861e-03,  2.0500954710685703e-04,  1.8425223343175262e-04,
       1.0861110383228870e-03,  4.4234479011584496e-03, -1.8523516026531252e-03,
      -1.0875087128278172e-03, -1.2838843998828526e-03,  9.2209129454743027e-04,
       5.5934200243103497e-04, -4.5973676346238362e-04, -1.7393681994482785e-03,
      -9.1657776736468559e-04,  1.6043261473213513e-04, -6.2240109209966640e-04,
      -2.4466597022790352e-03,  6.1650121465123083e-05, -2.3097016284274845e-03,
      -3.1776701723818536e-03,  1.3375937294404300e-04,  5.6087468939149739e-04,
      -1.6952794816291279e-03,  2.6871756876724966e-03, -1.8996902371860177e-04,
       4.3072889264212771e-04,  7.2261410749757129e-04,  3.7671625211815986e-05,
       1.4665326592621867e-03, -6.8454531560807586e-04,  1.1274391723861703e-03,
      -4.4127700260446180e-04,  1.7803517424559289e-03,  9.1272157946193548e-04,
      -3.8373024231821459e-03,  1.2236005655348023e-04, -2.8348985079790995e-03,
       9.3916950215424799e-04, -3.3379266936933863e-03,  1.4787556358457740e-03,
      -5.0181681374249015e-04, -1.8579712277841398e-04,  4.9535962637222342e-04,
       1.8543884349114205e-03,  2.7478057924776493e-03,  3.3052646587155882e-03,
      -1.5744426066769696e-03,  1.5882659017655860e-03,  8.7734447024789299e-04,
      -1.7907611740075598e-03, -1.1585649625737043e-03,  4.8061086669868387e-04,
      -1.8618584099290686e-03,  5.7188414048473988e-04, -2.7014010418316241e-03,
      -1.9663719293508642e-03,  5.1830425574771210e-04,  1.4821465870256179e-04,
      -1.4480350272120066e-03,  2.8157557088451192e-04,  3.1837965021630763e-03,
      -2.2829265081383025e-03, -2.2191627124696265e-03,  2.0240017084867034e-03,
      -1.0938324386652602e-03,  1.3550259304894279e-04,  2.4767390326809087e-03,
       5.2255301364578678e-05, -3.1419519940181027e-03,  6.6321542228463190e-03,
       3.2637158199539016e-03, -1.1265006421677470e-03,  1.4601322665743688e-03,
       2.1986608557843547e-03,  2.1965413487111071e-04,  3.8391354385855342e-03,
       8.5688685434025128e-04, -1.3177266953605691e-04, -2.0680263275225661e-03,
      -4.6073928606929666e-04,  1.3236678614748378e-03,  4.1420293195496118e-03,
      -3.6281154180847069e-04, -1.0291696062622541e-03, -3.8909669698282598e-03,
       8.6708168786002189e-04,  2.1059109303084422e-03,  4.5003797918722319e-04,
      -4.0448329139643771e-04, -9.4899442710483597e-04,  2.8434256922525219e-03,
      -1.9781185872501409e-03,  1.6407558571269784e-03,  5.1444971088759070e-04,
       7.2769157788001710e-04, -1.1964703363666262e-03,  2.8462204033280029e-03,
      -2.9380724891875229e-03,  4.8947948411620723e-03,  1.7985527108905612e-03,
      -2.8422153259941301e-03, -3.3585315117876254e-04,  2.7791937589599523e-03,
       1.1797675155209870e-03,  6.7022681535116777e-04,  2.5237094628121795e-03,
      -4.9537260004092233e-04,  6.9990539554923943e-04,  7.3122197826073771e-04,
      -1.9618276843438867e-04,  8.6385147841243175e-04,  1.9239589139237508e-03,
       9.3515027333201017e-04,  2.3216157272816351e-03, -1.2656961874392034e-03,
      -1.2373131359284630e-03,  1.2793210604142597e-03,  5.2676673302283180e-04,
       3.1714003500693310e-06,  1.4929631196458793e-03, -1.1322595921777398e-03,
      -2.3202516481046530e-03, -3.0085128116628100e-03, -8.3322155974273066e-04,
       2.7151136221372847e-03,  3.5844131930058562e-04, -4.3766898759395545e-04,
       2.8502244951498054e-03,  8.9593937419392704e-04, -4.1893077841336415e-03,
       8.5539987921079004e-05, -5.4172761663809647e-04, -1.8266866896162587e-03,
      -5.4410128232695095e-04,  2.1849049986899099e-03, -2.6937972358537661e-03,
      -2.4732404017917384e-04, -2.5709664848742476e-03, -1.7697048925071756e-03,
      -3.8501178292734675e-04, -1.8698528077764254e-03, -1.2330191028699038e-04,
       4.8202077722588488e-03,  3.4243971852509695e-03,  1.6189346351744651e-03,
       1.3885079720980236e-03, -7.8149715333148426e-04,  1.0324218631391701e-03,
      -1.1068052201978857e-03, -3.4469485912249969e-03,  1.3618471602343425e-03,
       2.1596956958868921e-03,  2.2364022898638831e-03,  1.0983538319402444e-03,
       1.3349025552493414e-04, -1.2215731735159474e-03, -1.2431967069248409e-04,
      -1.6084983970691439e-03, -3.1972666268102176e-04,  2.5147224958552311e-03,
       9.6076240428602468e-04,  3.0292161550216166e-03,  2.5248788126039606e-04,
       1.2351744467690524e-03, -1.8459082329818762e-03,  1.1310947717852087e-03,
       3.0923980380122965e-03, -1.6514396988354108e-03, -2.7492926899695328e-04,
       3.2013712657393200e-03, -1.7677476644524580e-03,  1.2478440897952356e-03,
      -3.0445211018070698e-04,  1.0359784362555370e-03,  3.2955693670871579e-03,
      -8.2100824736097537e-04, -1.7559393360733490e-03,  2.0543092249188512e-03,
       2.5695125913298908e-03,  2.4568948854445426e-03,  7.0193476914537237e-04,
      -2.2566402861922891e-04,  2.7529103341772704e-03,  1.8222782527657388e-03,
       2.7756977678825407e-03,  4.7846714277249711e-04, -2.7139488265557747e-03,
       7.0337130153948441e-04,  8.0329399387505689e-05,  2.5735647874260536e-03,
       2.7307878636903941e-03,  1.9780738001724070e-03,  3.3142856179447797e-03,
       5.9460848050779528e-04, -2.5723283847717705e-03,  6.6351146716791393e-04,
       5.7251852648327260e-05, -5.0171545408800853e-04,  3.3201362899349724e-04,
       8.9743115982701910e-04, -1.1266239577941380e-03, -2.1838131996667113e-03,
      -2.3809758926455872e-04, -2.0409060975378293e-03, -3.2359572880669195e-03,
      -2.6738789095071642e-03,  2.4798183855418822e-03, -2.5832781339452755e-03,
       2.7304587943507617e-03, -2.1167222846119440e-03,  1.0994020049537828e-03,
       1.9120560316001499e-03,  3.4102138830611026e-03, -3.0872731244508574e-03,
       2.9273950738729366e-03, -1.3838019852041627e-04,  4.2291313785729559e-04,
      -4.5781180127270858e-03,  6.5530473860932637e-04,  3.7166247653943685e-03,
      -4.3650688781145789e-04, -2.6588221471223269e-04,  1.5298810658161686e-03,
       2.0219053929643079e-03,  1.4637527690010144e-03,  1.0763369855308271e-03,
      -2.0401209724753787e-04, -3.6475686978202840e-04, -9.1767464274655387e-05,
       3.3800445059296234e-04,  4.0041138880789447e-03,  6.4907865095642329e-04,
       2.0087537190992028e-03,  5.2143355541131001e-04, -2.5537966486082991e-03,
       3.6050879403402881e-03, -6.9106592781361601e-04, -2.7993827976448009e-03,
       2.6764329403046125e-04,  2.8442432253202336e-03, -7.5915458385328674e-04,
       1.8847217961626553e-03,  1.0785574042140277e-03,  7.6690974093370943e-04,
      -3.6037185435984845e-03,  2.1737987793285078e-03, -5.1399509856317493e-04,
       1.3178277002701478e-03, -1.4450754093875912e-03, -1.6504958691275339e-03,
       1.4595374530161004e-03, -1.4535180355348106e-03,  7.4010933182507238e-04,
       2.0311090818258882e-03,  2.4561547416404859e-03,  9.1166330030072911e-04,
       1.7180522396261677e-03,  1.4920776157120019e-03, -5.0487437613382590e-04,
       2.0795612677914368e-03, -4.0496023602165540e-04,  2.6227851205328639e-03,
      -1.0506441149137769e-03, -2.0142113778720779e-03, -1.6245216913870048e-03,
       3.9226592801935743e-03, -7.1883572567233833e-04, -2.4510493978532606e-03,
      -2.6080660530953103e-03,  1.3719324621885786e-03,  2.5318295172421699e-03,
       1.7681256317432770e-03, -2.8148260992128392e-03,  1.2249057533193048e-03,
       9.6030537367832207e-04, -2.0062529037510731e-03, -3.0344078603015306e-04,
       6.9295431970730001e-03, -4.4433476376963408e-04,  1.6095312341850925e-03,
       5.1713503089137157e-03, -1.8460740375349663e-03,  2.2686063962000362e-03,
      -1.4905040502015431e-04,  3.0180685236583164e-03, -7.0312336914096925e-04,
       3.2627710822205626e-03, -1.5929210099597704e-03, -1.5987519105568164e-03,
       1.2231051018977989e-03, -2.1662930461679956e-04, -1.1070081829976652e-03,
      -4.2927709772218030e-03,  1.8015142681976060e-03, -1.9781543598554031e-03,
       1.0041669858306480e-06,  2.9896803963378317e-03,  9.3435636572354098e-04,
       2.0560354583182027e-03, -2.6851965321652562e-03, -2.7561631680221065e-03,
      -8.6875899227744746e-04, -1.0416071948520425e-04,  1.2040288818227268e-03,
      -2.2583316238889528e-03, -1.7469777972732234e-03,  1.0538039814686254e-03,
       2.5005058892342098e-03,  2.9587060103927459e-03, -1.4144670643269916e-03,
       3.6121462024562916e-04, -3.4852610758247891e-03,  6.6436066279190178e-04,
      -4.1361798256014685e-03, -1.9754143885442276e-03, -9.7693631405958809e-04,
      -2.7287020610596642e-04, -2.0328000083642468e-03, -2.3909137063062250e-03,
       1.4371789516868985e-03,  8.5863191742490156e-04, -3.7866252132931497e-03,
       3.1398989764748332e-03,  1.2328732034706494e-03,  3.1428146897675853e-03,
       1.3686708140670941e-03,  3.4345198389163726e-03, -1.8009019440431719e-03,
      -2.1490314348572557e-03,  4.9749201617061579e-03, -2.7206174510175990e-03,
       4.8522398628127626e-03, -1.1434923015395224e-03,  1.7264866735682059e-03,
       2.3573996964604028e-03, -1.8667345709125450e-03,  2.0251198904242796e-03,
      -2.2660034679000221e-03, -4.6170215680998196e-03, -9.6162963136995142e-04,
      -1.3532977598216809e-03, -2.6261355875734369e-03,  9.5647421342355424e-04,
      -1.8226829641112166e-03, -1.3628565137894756e-03, -2.1195490537164241e-04,
       1.8196406577282131e-04,  2.9166398325335633e-03, -3.6857562753056352e-03,
      -9.6157457553101920e-04, -5.9615502710848572e-04, -1.9478234716327895e-03,
       1.2227935604909573e-03,  2.1037373443485901e-03, -1.0356736219702906e-03,
      -6.5202772631229059e-04, -6.7002158060801742e-04, -1.9228789970562935e-03,
       2.7922819259193700e-03,  8.9845331238944104e-04, -2.7300453894143724e-04,
      -1.9898784324107219e-03, -2.7953358410380815e-03,  1.6294017967716495e-03,
       3.7212852797070860e-03,  1.5818859484923568e-04,  1.1877491305160809e-03,
       2.9773538203415623e-03, -1.6192253785231945e-03,  2.6779492814982984e-03,
       2.5833348321261583e-04,  6.6592179305878562e-04, -1.9019030300518919e-03,
       5.3744517405430808e-04, -3.4000063946175644e-03, -2.8977334589051188e-04,
      -1.1577693566876370e-03, -4.6104790709733728e-03,  6.5192651901517745e-04,
       2.1821511608371238e-03, -5.9206777321549191e-05, -5.8533368449464873e-04,
       1.8714375942195834e-03,  2.0049874661558911e-04,  2.6159361746945463e-03,
       1.0662942330915629e-03, -2.3858026755313160e-03,  1.2618691326554924e-03,
      -3.3383163691218117e-03, -1.2419472728155280e-03,  1.0158132732539140e-03,
       1.2053853115125306e-03,  7.6890554683551086e-04, -2.5890429488857397e-03,
      -3.0535052076961691e-03, -2.0526621360152496e-03, -1.2541488123099730e-03,
       4.1928188939233816e-03,  1.3765987817130467e-03, -5.2486479181803561e-04,
       3.4059831786262609e-03,  9.4840525408587770e-04, -4.9124010160642387e-03,
      -3.6227478385070754e-03, -2.6081273949091891e-03,  5.2669789543579454e-03,
      -1.0161190382277669e-03, -4.3831595594452763e-04, -1.3025356365476358e-03,
       1.1066935851096284e-04, -6.8042859390536950e-04, -3.8887296875019997e-04,
      -4.5576727165143067e-04, -3.2081593625891908e-03,  3.6048961007557031e-03,
      -3.0199781173198500e-03, -2.2210297257891213e-03, -2.5835924565757798e-03,
      -6.0070925756061341e-04, -1.8253704578578573e-03,  2.3355607485423951e-03,
       2.3950898982277838e-03, -3.6267617356803513e-03,  4.6346143953117092e-03,
      -2.0079344919455740e-03, -2.9796990722199141e-04, -1.2914793602680262e-03,
       2.3173148831356072e-03,  1.3524541606325128e-03, -2.7413661576095143e-03,
      -2.6435948250061272e-03,  1.1469173222287184e-03, -2.2658203861674088e-03,
      -1.1135058458321867e-03,  3.3082640056365654e-03, -2.7490846444963701e-03,
      -4.2893709913273956e-04, -2.9010467858744129e-03,  4.3080848622595736e-04,
      -1.1928245208173123e-03,  1.2229804118833486e-03,  4.1560555072808932e-04,
       3.1660428661748298e-03,  2.9462723542924320e-03,  1.4776743052865476e-03,
      -2.1233929396510870e-03, -7.4243510106238454e-04, -3.9947073082701546e-04,
      -2.3119807007679900e-03,  2.3860126260452021e-03, -1.0711206008000244e-03,
       4.9221600347715202e-04,  1.4876828053222890e-03,  1.2236417807660173e-03,
      -1.9076715438186198e-03, -5.0361379236778562e-04, -2.0878577906363045e-03,
      -8.6745164979855566e-04,  6.3861624023806810e-04,  1.7356548629323667e-03,
       1.5253308192362733e-03,  2.0741586582983546e-03, -9.1104207198791068e-04,
       6.8408985141230440e-04, -3.8761060300412817e-04,  2.7038064473001804e-04,
      -3.1538121823465612e-03, -5.8017143189699623e-04,  1.0546228868133119e-03,
       4.2360726831307185e-04,  1.3864395216639700e-03,  3.0956457038191788e-04,
      -2.4222330057188947e-03,  1.4265302303108900e-03, -5.8484094455438804e-04,
       2.4849619113972176e-03, -3.7913894977609705e-03,  9.1229682491160080e-05,
       1.6845523759446408e-03, -1.7203474114828260e-03,  5.1718951556884250e-03,
       3.5501506290569855e-03,  3.9637984404567464e-04,  1.0492322194301869e-03,
      -6.2178157675070748e-04, -4.0655211678018001e-03, -6.2953987497805218e-04,
      -3.5170680636194767e-04,  1.7669285168262386e-05,  5.2303523118933616e-04,
      -8.9274032233689984e-04,  7.5768604139601046e-04,  4.3833538446639096e-03,
       2.6771056303373053e-04, -2.0860301120967494e-03, -7.5665027110882364e-04,
       1.8221919959649519e-03, -3.0075413547003562e-03, -2.1695671199456192e-04,
       5.7229024239067615e-04,
    ]
}     