// [[file:../gchemol-core.note::*imports][imports:1]]
use serde::*;

use gchemol_graph::*;
use gut::prelude::*;

use bimap::BiHashMap;
use gchemol_lattice::Lattice;

use crate::atom::*;
use crate::bond::*;
use crate::element::*;
use crate::property::PropertyStore;
// imports:1 ends here

// [[file:../gchemol-core.note::*base][base:1]]
type MolGraph = NxGraph<Atom, Bond>;

/// Molecule is the most important data structure in gchemol, which repsents
/// "any singular entity, irrespective of its nature, used to concisely express
/// any type of chemical particle that can exemplify some process: for example,
/// atoms, molecules, ions, etc. can all undergo a chemical reaction". Molecule
/// may have chemical bonds between atoms.
///
/// Reference
/// ---------
/// 1. http://goldbook.iupac.org/M03986.html
/// 2. https://en.wikipedia.org/wiki/Molecular_entity
///
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Molecule {
    /// Arbitrary property stored in key-value pair. Key is a string type, but
    /// it is the responsibility of the user to correctly interpret the value.
    pub properties: PropertyStore,

    /// Crystalline lattice for structure using periodic boundary conditions
    pub lattice: Option<Lattice>,

    /// Molecular name.
    pub(crate) name: String,

    /// core data in graph
    pub(crate) graph: MolGraph,

    /// mapping: Atom serial number <=> graph NodeIndex
    pub(crate) mapping: BiHashMap<usize, NodeIndex>,
}

/// Methods for internal uses
impl Molecule {
    /// get internal node index by atom sn.
    pub(crate) fn node_index(&self, sn: usize) -> NodeIndex {
        *self.get_node_index(sn).expect(&format!("invalid atom sn: {}", sn))
    }

    /// get internal node index by atom sn.
    pub(crate) fn get_node_index(&self, sn: usize) -> Option<&NodeIndex> {
        self.mapping.get_by_left(&sn)
    }

    /// get atom sn  by internal node index.
    pub(crate) fn atom_sn(&self, n: NodeIndex) -> usize {
        *self
            .mapping
            .get_by_right(&n)
            .expect(&format!("invalid NodeIndex: {:?}", n))
    }

    /// Removes atom sn from mapping and returns the associated NodeIndex.
    pub(crate) fn remove_atom_sn(&mut self, sn: usize) -> Option<NodeIndex> {
        self.mapping.remove_by_left(&sn).map(|(_, n)| n)
    }
}
// base:1 ends here

// [[file:../gchemol-core.note::*constructor][constructor:1]]
/// `Molecule` constructors
impl Molecule {
    /// Create a new empty molecule with specific name
    pub fn new(name: &str) -> Self {
        Molecule {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Build a molecule from iterator of atoms associated with serial numbers
    /// from 1.
    pub fn from_atoms<T>(atoms: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Atom>,
    {
        let mut mol = Self::default();
        for (i, a) in atoms.into_iter().enumerate() {
            mol.add_atom(i + 1, a.into());
        }

        mol
    }

    /// Build `Molecule` from raw graph struct.
    pub fn from_graph(graph: MolGraph) -> Self {
        let mut mol = Self {
            graph,
            ..Default::default()
        };

        // create serial number mapping
        let nodes: Vec<_> = mol.graph.node_indices().collect();
        for (sn, n) in (1..).zip(nodes.into_iter()) {
            mol.mapping.insert_no_overwrite(sn, n).expect("from graph failure");
        }

        mol
    }
}
// constructor:1 ends here

// [[file:../gchemol-core.note::*basic][basic:1]]
/// Core methods
impl Molecule {
    /// Add atom `a` into molecule. If Atom numbered as `a` already exists in
    /// molecule, then the associated Atom will be updated with `atom`.
    pub fn add_atom(&mut self, a: usize, atom: Atom) {
        if let Some(&n) = self.mapping.get_by_left(&a) {
            self.graph[n] = atom;
        } else {
            let n = self.graph.add_node(atom);
            self.mapping.insert_no_overwrite(a, n).expect("add atom failure");
        }
    }

    /// Remove Atom `a` from `Molecule`.
    ///
    /// Return the removed Atom on success, and return None if Atom `a` does not
    /// exist.
    pub fn remove_atom(&mut self, a: usize) -> Option<Atom> {
        if let Some(n) = self.remove_atom_sn(a) {
            self.graph.remove_node(n)
        } else {
            None
        }
    }

    /// Return the number of atoms in the molecule.
    pub fn natoms(&self) -> usize {
        self.graph.number_of_nodes()
    }

    /// Return the number of bonds in the molecule.
    pub fn nbonds(&self) -> usize {
        self.graph.number_of_edges()
    }

    /// Add `bond` between Atom `a` and Atom `b` into molecule. The existing
    /// Bond will be replaced if Atom `a` already bonded with Atom `b`.
    ///
    /// Panic if the specified atom `a` or `b` does not exist
    ///
    pub fn add_bond(&mut self, a: usize, b: usize, bond: Bond) {
        let na = self.node_index(a);
        let nb = self.node_index(b);
        self.graph.add_edge(na, nb, bond);
    }

    /// Remove the bond between atom `a` and atom `b`.
    ///
    /// Returns the removed `Bond` on success
    ///
    /// Panic if the specified atom `a` or `b` does not exist
    pub fn remove_bond(&mut self, a: usize, b: usize) -> Option<Bond> {
        let na = self.node_index(a);
        let nb = self.node_index(b);
        self.graph.remove_edge(na, nb)
    }

    /// Remove all atoms and bonds. To remove bonds only, see [unbound](#method.unbound) method.
    pub fn clear(&mut self) {
        self.mapping.clear();
        self.graph.clear();
    }

    /// Iterate over atoms ordered by serial numbers.
    pub fn atoms(&self) -> impl Iterator<Item = (usize, &Atom)> {
        // sort by atom serial numbers
        self.serial_numbers().map(move |sn| {
            let n = self.node_index(sn);
            let atom = &self.graph[n];
            (sn, atom)
        })
    }

    /// Iterate over bonds in arbitrary order.
    pub fn bonds(&self) -> impl Iterator<Item = (usize, usize, &Bond)> {
        self.graph.edges().map(move |(u, v, bond)| {
            let sn1 = self.atom_sn(u);
            let sn2 = self.atom_sn(v);
            (sn1, sn2, bond)
        })
    }

    /// Iterate over atom serial numbers in ascending order. Serial number is an
    /// unsigned integer (1-based, traditionally) for accessing `Atom` in
    /// `Molecule`
    pub fn serial_numbers(&self) -> impl Iterator<Item = usize> {
        self.mapping.left_values().copied().sorted()
    }

    #[cfg(feature = "adhoc")]
    /// A shorter alias to [serial_numbers](#method.serial_numbers) method.
    pub fn numbers(&self) -> impl Iterator<Item = usize> + '_ {
        self.serial_numbers()
    }

    #[cfg(not(feature = "adhoc"))]
    #[deprecated(since = "0.0.40", note = "please use atomic_numbers instead")]
    /// Return an iterator over atomic numbers of atoms.
    pub fn numbers(&self) -> impl Iterator<Item = usize> + '_ {
        self.atomic_numbers()
    }

    /// Iterate over atom symbols ordered by serial numbers.
    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.atoms().map(move |(_, atom)| atom.symbol())
    }

    /// Iterate over atomic numbers.
    pub fn atomic_numbers(&self) -> impl Iterator<Item = usize> + '_ {
        self.atoms().map(move |(_, atom)| atom.number())
    }

    /// Iterate over atom positions ordered by serial numbers.
    pub fn positions(&self) -> impl Iterator<Item = Point3> + '_ {
        self.atoms().map(move |(_, atom)| atom.position())
    }

    /// Return the name of the molecule, which is typpically modified for safely
    /// storing in various chemical file formats.
    pub fn title(&self) -> String {
        let tlines: Vec<_> = self.name.lines().collect();
        if tlines.is_empty() {
            "untitled".to_owned()
        } else {
            tlines[0].trim().to_owned()
        }
    }

    #[cfg(feature = "adhoc")]
    /// Return a reference to internal Molecule Graph struct.
    pub fn graph(&self) -> &NxGraph<Atom, Bond> {
        &self.graph
    }

    #[cfg(feature = "adhoc")]
    /// Return mut access to internal Molecule Graph struct.
    pub fn graph_mut(&mut self) -> &mut NxGraph<Atom, Bond> {
        &mut self.graph
    }
}
// basic:1 ends here

// [[file:../gchemol-core.note::*edit][edit:1]]
/// Edit `Molecule`
impl Molecule {
    /// Read access to atom by atom serial number.
    pub fn get_atom(&self, sn: usize) -> Option<&Atom> {
        self.get_node_index(sn).map(|&n| &self.graph[n])
    }

    /// Mutable access to atom by atom serial number.
    pub fn get_atom_mut(&mut self, sn: usize) -> Option<&mut Atom> {
        // self.get_node_index(sn).map(move |&n| &mut self.graph[n])
        if let Some(&n) = self.get_node_index(sn) {
            Some(&mut self.graph[n])
        } else {
            None
        }
    }

    /// Read access to bond by a pair of atoms. Return None if there is no bond
    /// between Atom `sn1` and Atom `sn2`.
    pub fn get_bond(&self, sn1: usize, sn2: usize) -> Option<&Bond> {
        if let Some(&n1) = self.get_node_index(sn1) {
            if let Some(&n2) = self.get_node_index(sn2) {
                return Some(&self.graph[(n1, n2)]);
            }
        }
        None
    }

    /// Mutable access to bond by a pair of atoms.
    pub fn get_bond_mut(&mut self, sn1: usize, sn2: usize) -> Option<&mut Bond> {
        if let Some(&n1) = self.get_node_index(sn1) {
            if let Some(&n2) = self.get_node_index(sn2) {
                return Some(&mut self.graph[(n1, n2)]);
            }
        }
        None
    }

    /// Set atom position.
    ///
    /// Panic if atom `sn` does not exist.
    pub fn set_position<P: Into<Vector3f>>(&mut self, sn: usize, position: P) {
        let atom = self.get_atom_mut(sn).expect("invalid atom serial number");
        atom.set_position(position);
    }

    /// Set atom symbol.
    ///
    /// Panic if atom `sn` does not exist.
    pub fn set_symbol<S: Into<AtomKind>>(&mut self, sn: usize, sym: S) {
        let atom = self.get_atom_mut(sn).expect("invalid atom serial number");
        atom.set_symbol(sym);
    }

    /// Add a list of atoms into molecule.
    pub fn add_atoms_from<T, P>(&mut self, atoms: T)
    where
        T: IntoIterator<Item = (usize, P)>,
        P: Into<Atom>,
    {
        for (n, a) in atoms {
            self.add_atom(n, a.into());
        }
    }

    /// Set molecular title.
    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.name = title.as_ref().to_owned();
    }

    /// Add a list of bonds into molecule.
    pub fn add_bonds_from<T>(&mut self, bonds: T)
    where
        T: IntoIterator<Item = (usize, usize, Bond)>,
    {
        for (u, v, b) in bonds {
            self.add_bond(u, v, b);
        }
    }

    /// Set positions of atoms in sequential order.
    pub fn set_positions<T, P>(&mut self, positions: T)
    where
        T: IntoIterator<Item = P>,
        P: Into<Vector3f>,
    {
        for (sn, p) in self.serial_numbers().zip(positions.into_iter()) {
            let atom = self.get_atom_mut(sn).unwrap();
            atom.set_position(p);
        }
    }

    /// Update positions of atoms in sequential order, with freezing coordinates
    /// ignored.
    pub fn update_positions<T, P>(&mut self, positions: T)
    where
        T: IntoIterator<Item = P>,
        P: Into<Vector3f>,
    {
        for (sn, p) in self.serial_numbers().zip(positions.into_iter()) {
            let atom = self.get_atom_mut(sn).unwrap();
            atom.update_position(p);
        }
    }

    /// Set positions of specified atoms
    pub fn set_positions_from<T, P>(&mut self, selected_positions: T)
    where
        T: IntoIterator<Item = (usize, P)>,
        P: Into<Vector3f>,
    {
        for (i, p) in selected_positions {
            self.set_position(i, p);
        }
    }

    /// Set element symbols
    pub fn set_symbols<T, S>(&mut self, symbols: T)
    where
        T: IntoIterator<Item = S>,
        S: Into<AtomKind>,
    {
        for (sn, sy) in self.serial_numbers().zip(symbols.into_iter()) {
            let atom = self.get_atom_mut(sn).unwrap();
            atom.set_symbol(sy);
        }
    }

    /// Remove atoms from .. (unimplemented)
    pub fn remove_atoms_from(&mut self) {
        unimplemented!()
    }

    /// Remove bonds from .. (unimplemented)
    pub fn remove_bonds_from(&mut self) {
        unimplemented!()
    }
}
// edit:1 ends here

// [[file:../gchemol-core.note::*test][test:1]]
#[test]
fn test() {
    let mut mol = Molecule::new("test");

    for i in 0..5 {
        mol.add_atom(i, Atom::default());
    }
    assert_eq!(mol.natoms(), 5);

    mol.add_bond(1, 2, Bond::single());
    mol.add_bond(2, 3, Bond::double());
    assert_eq!(mol.nbonds(), 2);
    mol.add_bond(2, 1, Bond::single());
    assert_eq!(mol.nbonds(), 2);

    for (i, a) in mol.atoms() {
        dbg!((i, a.symbol()));
    }

    // set title
    mol.set_title("new mol");
    mol.set_title(format!("Molecule: {}", 4));
}
// test:1 ends here
