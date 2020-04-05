use super::*;

use std::time::{Duration};

#[test]
fn collide_two_paths() {
    // Plus sign, combined into a path
    let edits = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAiB+2FAAodjLHRF9PA8BAcNj5P1EA4AAAAAAAAAAAAAAGAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAANAAAAAAAAAAAAAAEAAAAAAAAlXAaIAEAAAAAAAA2MAsBACAAAAAAAAXPAGCACAAA8PAAArlAbEACAAAAAAAAGWAUCADAAAAAAAAbYA5BABAAAAAAAAVaArBABAAAAAAAAocAsBABAAAAAAAAieAQBABAAAAAAAAOgADBAAAAAAAAAA5hA1AAAAAAAAAAAXjAbAAAAAA4PAAAM3Cs9PAAAAAAAAAAkAU+PAAAA8PAAA8iAI+PAAAAAAAIAfhAU+PAAAAAAAAAyfAU+PAAAA4PAAADdAj+PAAAAAAAAADxA28P//PAAAAAAmvAv6P8/PA8PAAAQJAw+P0/PAAAAAA9GAi+P4/PA4PAAAbEA9+P3/PAAAAAAhCA9+Pw/PAAAAAA1AAL/Pn/PAAAAAAAAAl/PZ/PAAAAAA69PAAAF/PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAjBAAoZmS0QAA4MzFIRt9PAsBAYNJBAB/PQAAAAAAAAAAAAAAIAAAAAAAAAAAAAADAAAAAAAAAAAAAABAAAAAAAAAAAAAACAAAAAAAAAAAAAALAAAAAAAAAAAAAAEAAAAAAAAoAAbkPDAAAAAAAANAAUyPCAAAAAAAAAAALvPCAAAAAAAAAAAKrPCAAAAAAEAi+P9mPCAAAAAAAAmzPkbOBAAAAAAAAU6PNYPDAAAAAAIA65PiWPAAAAAAAEAU6PhWPAAAAAAAAAm7PYXPAAAAAAAEAD9PEZPAAAAAAAAA9+PYbPAAAAAAAIAAAA6dPAAAAAAAAAsBA8CPAAAAAAAAAUCAflPAAAAAAAEAhCAAoPAAAAAAAAAGCAhqPAAAAAAAIAHCA3sPAAAAAAAAA4BAKvPAAAAAAAAAeBAexPAAAAAAAEAeBAYzPAAAAAAAAAQBAs1P//PAAAAAAXDA6tP+/PAAAAAADBAAAAu/PAAAAEAQBAhCA0/PAAAAAA2AAXDAq/PAAAAAAQBAGKAE/PAAAAAA
        EB+Dj
    ";

    // Run the edits
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, edits);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 2);
}

#[test]
fn collide_three_paths_by_adding_to_existing_collision() {
    // Collide two paths then add an extra one to the collision
    let edits = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAoBAAoZmBwQAAoZGMIRa8PAACA4N8BAL/PRBAAAAAAAAAAAAAKAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAEAAAAAAAAAAAAAAEAAAAAAAAaAAslPCAAAAAAAAAAAA0PCAAAAAAIAAAA5xPDAAAAAAAAI+PbEPDAAAAAAAAs9PslPKAAAAAAEA28P5hPCAAAAAAAAc8PwePBAAAAAAAAA8PAcPBAAAAAAEAm7PeZPBAAAAAAAAK7PzXPAAAAAAAAAL7PvWPAAAAAAAIAEtPJbNAAAAAAAAAO8PVaPAAAAAAAMAp8PpcPAAAAAAAAAD9PXfPAAAAAAAEAf9PviPAAAAAAAAAs9PslPAAAAAAAEA59P2oPAAAAAAAAAL7PsJPAAAAAAAIAw+PiyPAAAAAAAAAZ/P10PAAAAAAAEAl/P+2PAAAAAAAAAz/Po4P//PAAAAAAz/Pj6P//PAAAAAAAAAO8P+/PAAAAIAAAAr9P9/PAAAAAAAAAHGA8/PAAAAAAAAAuGAL/PAAAAAAAAAsFAf/PAAAAAAAAADNAD/PAAAAAA
        EB+Dj
        LBPtAAAAAA*+HIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+IAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+GAiBNnEAAomZyHRl9PAMCAAO6AAS9PNAAAAAAAAAAAAAAAAAAAAAAAAAAAAA9/PAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAeVA0/PGAAAAAAAApIAaAAOAAAAAAAATKA2AAFAAAAAAAApMADBAEAAA8PAAA8OArBACAAAAAAAACRA5BADAAAAAAAA8SAUCACAAAAAAAAoUAhCACAAA4PAAAStARFACAAAAAAAAoYAhCAEAAAAAAAAeZAGCACAAA8PAAAvaA5BABAAAAAAAAlbAeBAAAAAAAAAAOcAQBAAAAAAAAAApcA1AAAAAAAAAAA1cAbAAAAAA8PAAA1cAaAAAAAAAAAAARRBNAAAAAAAAAAAKXAAAAAAAA4PAAAoUA9+P+/PAAAAAA5RAU+P9/PAAAAAAKPAV+P9/PA8PAEACNAH+P8/PAAAAAA9KAU+P5/PAAAAAARJAV+P2/PAAAAAAXHAH+Pu/PAAAAAADFAi+Ph/PAAAAAAXDAI+P7+PAAAAAA
        EB+Gj
        LBPtAAAAAAS+JAhBAkDAAYzs6FRC9PAACAgO9AAf9PmAAAAAAAAAAAAAALAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAFAAAAAAAAAAAAAAEAAAAAAAAAAAAAADAAAAAAAAvOAHSADAAAAAAAA1IAhCAGAAAAAAAAAYARFACAAAAAAAAXPAuCAGAAAAAAAAeRA9CADAAAAAAAANUA9CADAAAAAAAAUWAJDADAAAAAAAApYAYDACAAA8PAAAvaAkDABAAAAAAAAU6AvGABAAAAAAAAXfA7CACAAA8PAAAOgAhCAAAAAAAAAADhAHCAAAAAAAAAArhAdBAAAAAAAAAARhAQBAAAAAAAAAAbgAoAAAAAA4PAAA8eAAAAAAAAAAAAAbcAAAAAAAAAAAAAzDBL/P//PAAAAAAeRAK/Pz/PA8PAAAlPAZ/P4/PAAAAAAHOAl/P3/PA4PAAA1MAAAAw/PAAAAAADJAAAAO/PAAAAAA
        EB+Jj
    ";

    // Run the edits
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, edits);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 3);
}

#[test]
fn collide_three_paths_all_at_once() {
    // Draw two lines and join them to make an 'H' (which should all collide into one)
    let two_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAsBaqFAAoZOEIRN9PAMCAAONDA4AAcAAAAAAAAAAAAAAKAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAFAAAAAAAAAAAAAAHAAAAAAAAAAAAAAFAAAAAAAAnAAUqPEAAAAAAAA9+PUmPCAAAAAAAA69PYvPEAAAAAAEAf9PEtPCAAAAAAAA28PiqPCAAAAAAAA38PcoPCAAAAAAAA28PHmPCAAAAAAAAD9PAkPCAAAAAAEAF9PiiPBAAAAAAAAU6PcAPAAAAAAAAAR9PhePBAAAAAAIAR9PRdPAAAAAAAAAS9P2cPAAAAAAAEAD9PbcPAAAAAAAAAD9PbcPAAAAAAAAAF9PDdPAAAAAAAAAD9P6dPAAAAAAAIAD9P9ePAAAAAAAAAv6PAEPAAAAAAAAAs9PElPAAAAAAAEA69PKnPAAAAAAAEAz7PzTPAAAAAAAAA69PbsPAAAAAAAIA59PVuPAAAAAAAAAV+PBwPAAAAAAAAAU+P5xP//PA8PAAA69PRpP//PAAAAEAl/PpwP9/PAAAAAAAAA96P4/PAAAAIAAAAp8P4/PAAAAAA1AAU+P0/PAAAAAA5BAAAAs/PAAAAAAsBAAAAa/PAAAAAAJDAeFA3+PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAnBAAYzMgzQAAIAg0HR48PAACAEOYDAX+PlAAAAAAAAAAAAAALAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAGAAAAAAAAAAAAAAKAAAAAAAAAAAAAAIAAAAAAAAAAAAAAFAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAA//PAAAAAAoAAw+PAAAAAAAAANAAi6PDAAAAAAAAMAAB4PDAAAAAAAAAAAKnPDAAAAAAAAAAA8uPEAAAAAAAA9+PBsPBAAAAAAEAV+PRpPBAAAAAAAAf9P6lPBAAAAAAAA28P9iPCAAAAAAAAO8PlfPBAAAAAAIAz7P2cPBAAAAAAAAY3PyzOBAAAAAAEAm3P4xOBAAAAAAAAO8PiaPAAAAAAAIAc8PNcPAAAAAAAAAR9PhePAAAAAAAEAR9P3gPAAAAAAAAA69PXjPAAAAAAAEAH+P6lPAAAAAAAAAi6P4tOAAAAAAAAAL/PExP+/PAAAAIAL/PKzPAAAAAAAEAK/Pf1P//PAAAAAAZ/PJ3P//PAAAAAAl/Ps5P+/PAAAAAAm/P07P+/PAAAAIAl/PCRA8/PAAAAAA
        EB+Dj
    ";
    let join_lines = "
        LBPtAAAAAAS+GAiBYtCAA4QggGRZ8PAICAQOAAAOBATBAAAAAAAAAAAAALAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+/PAAAAAAAAAAAAAAAAAAAAAAAAAAAHAAAAAAAAAAAAAAHAAAAAAAAbYAz/PGAAAAAAAA1IAAAADAAAAAAAAhKAAAAEAAAAAAAAoMAAAADAAAAAAAABcB5BACAAAAAAAANYANAALAAA4PAAAUaAAAABAAAAAAAAAcAAAAAAAAAAAAAsdAMAAAAAAAAAAAXfANAAAAAA8PAAAtxCAAAAAAAAAAAADlAAAABAAA8PAEAAkAAAAAAAAAAAAA5hAAAAAAAA4PAAAyfAAAAAAAAAAAAABoBL7PAAAAAAAAAEVAL/PAAAA8PAAAXTAY/PAAAAAAAAAfRAz/P//PAAAAAAyPAAAA//PAAAAAAHOAAAAAAAAAAAAAOMAAAA//PAAAAAAGKAbAA//PAAAAAAGWAeBA+/PAAAAAANAAAAAL/PAAAAAAAAAAAAW/PAAAAAA
        EB+Gj
    ";

    // The two lines on either side of the 'H'
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, two_lines);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // These don't join together
    assert!(elements.len() == 2);

    // The cross line that forms the 'H' shape
    perform_serialized_edits(&mut animation, join_lines);

    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Everything joined into one element now
    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 3);
}
