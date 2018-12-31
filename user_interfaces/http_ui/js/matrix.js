'use strict';

//   __ _                    _       _
//  / _| |___ ___ _ __  __ _| |_ _ _(_)_ __
// |  _| / _ \___| '  \/ _` |  _| '_| \ \ /
// |_| |_\___/   |_|_|_\__,_|\__|_| |_/_\_\

/* exported flo_matrix */

let flo_matrix = (function (){
    ///
    /// Computes the determinant of a 2x2 matrix
    ///
    let det2 = (matrix) => {
        return (matrix[0]*matrix[3] - (matrix[1]*matrix[2]));
    };
    
    ///
    /// Computes the minor of an element in a 3x3 matrix
    ///
    let minor3 = (matrix, col, row) => {
        let small_matrix = [];

        let pos = 0;
        for (let y=0; y<3; ++y) {
            if (y !== row) {
                for (let x=0; x<3; ++x) {
                    if (x !== col) {
                        small_matrix.push(matrix[pos]);
                    }
                    ++pos;
                }
            } else {
                pos+=3;
            }
        }

        return det2(small_matrix);
    };

    ///
    /// Computes the cofactor of an element in a 3x3 matrix
    ///
    let cofactor3 = (matrix, col, row) => {
        let minor   = minor3(matrix, col, row);
        let sign    = (col&1) ^ (row&1);

        if (sign !== 0) {
            return -minor;
        } else {
            return minor;
        }
    };

    ///
    /// Computes the determinant of a 3x3 matrix
    ///
    let det3 = (matrix) => {
        return matrix[0]*cofactor3(matrix, 0,0) + matrix[1]*cofactor3(matrix, 1,0) + matrix[2]*cofactor3(matrix, 2,0);
    };

    ///
    /// Inverts a 3x3 matrix
    ///
    let invert3 = (matrix) => {
        let cofactors = [
            cofactor3(matrix, 0,0), cofactor3(matrix, 1,0), cofactor3(matrix, 2,0),
            cofactor3(matrix, 0,1), cofactor3(matrix, 1,1), cofactor3(matrix, 2,1),
            cofactor3(matrix, 0,2), cofactor3(matrix, 1,2), cofactor3(matrix, 2,2)
        ];

        let det     = matrix[0]*cofactors[0] + matrix[1]*cofactors[1] + matrix[2]*cofactors[2];
        let inv_det = 1.0/det;

        return [
            inv_det * cofactors[0], inv_det * cofactors[3], inv_det * cofactors[6],
            inv_det * cofactors[1], inv_det * cofactors[4], inv_det * cofactors[7],
            inv_det * cofactors[2], inv_det * cofactors[5], inv_det * cofactors[8]
        ];
    };

    ///
    /// Multiplies a 3x3 matrix with a 3D vector
    ///
    let mulvec3 = (matrix, vector) => {
        return [
            matrix[0]*vector[0] + matrix[1]*vector[1] + matrix[2]*vector[2],
            matrix[3]*vector[0] + matrix[4]*vector[1] + matrix[5]*vector[2],
            matrix[6]*vector[0] + matrix[7]*vector[1] + matrix[8]*vector[2]
        ];
    };

    return {
        det3:       det3,
        invert3:    invert3,
        mulvec3:    mulvec3
    };
})();
